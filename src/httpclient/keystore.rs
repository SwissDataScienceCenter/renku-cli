use std::{path::PathBuf, sync::Arc};

use db_keystore::{DbKeyStore, DbKeyStoreConfig};
use directories::ProjectDirs;
use keyring_core::{self, CredentialStore};
use log;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::data::renku_url::RenkuUrl;

use super::auth::Response;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error creating a keystore: {}", source))]
    KeystoreCreate { source: keyring_core::Error },

    #[snafu(display("Cannot write file: {}", source))]
    WriteFile { source: std::io::Error },

    #[snafu(display("Error creating a keystore entry: {}", source))]
    BuildEntry { source: keyring_core::Error },

    #[snafu(display("Error writing the secret: {}", source))]
    WriteSecret { source: keyring_core::Error },

    #[snafu(display("Error reading the secret: {}", source))]
    ReadSecret { source: keyring_core::Error },

    #[snafu(display("Error converting the token into json: {}", source))]
    ToJson { source: serde_json::Error },

    #[snafu(display("Error decoding token data: {}", source))]
    FromJson { source: serde_json::Error },
}

/// Sets the global default keyring store for the underlying keyring library.
pub fn set_default_global_keyring_store() -> Result<bool, Error> {
    if keyring_core::get_default_store().is_some() {
        log::debug!("A keystore has already been configured");
        return Ok(false);
    }
    let ks = KeyringStore::create_underlying_keyring()?;
    keyring_core::set_default_store(ks);
    Ok(true)
}

/// Keystore api used with the renku http client.
pub trait Keystore {
    fn write_token(&self, token: &Response) -> Result<(), Error>;
    fn read_token(&self) -> Result<Option<Response>, Error>;
    fn clear(&self) -> Result<(), Error>;
}

pub struct KeyringStore {
    renku_url: RenkuUrl,
    store: Arc<keyring_core::CredentialStore>,
}

const FORCE_KEYSTORE: &str = "RENKU_CLI_KEYSTORE";

/// A enum to let the user overwrite the default keystore.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KeystorePreference {
    Default,
    #[cfg(target_os = "linux")]
    LinuxKeyUtils,
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    DBus,
    File,
    Memory,
}

impl KeystorePreference {
    pub fn from_env() -> KeystorePreference {
        match std::env::var(FORCE_KEYSTORE).ok() {
            None => KeystorePreference::Default,
            Some(name) => Self::from_str(&name),
        }
    }
    pub fn from_str(name: &str) -> KeystorePreference {
        let p_name = format!("\"{}\"", name);
        let pref = serde_json::from_str::<KeystorePreference>(&p_name);
        match pref {
            Ok(p) => p,
            Err(msg) => {
                log::warn!("Invalid keystore preference '{}': {}", name, msg);
                KeystorePreference::Default
            }
        }
    }
}

impl KeyringStore {
    fn create_underlying_keyring() -> Result<Arc<keyring_core::CredentialStore>, Error> {
        match KeystorePreference::from_env() {
            KeystorePreference::File => {
                log::info!("Using file keystore as requested.");
                get_db_keystore()
            }
            #[cfg(target_os = "linux")]
            KeystorePreference::LinuxKeyUtils => {
                use linux_keyutils_keyring_store::Store as KernelStore;

                log::info!("Using linux keyutils keyring as requested.");
                let cs: Arc<CredentialStore> = KernelStore::new().context(KeystoreCreateSnafu)?;
                Ok(cs)
            }
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            KeystorePreference::DBus => {
                use zbus_secret_service_keyring_store::Store as ZBusStore;

                log::info!("Using an dbus secret service as requested.");
                let cs: Arc<CredentialStore> = ZBusStore::new().context(KeystoreCreateSnafu)?;
                Ok(cs)
            }
            KeystorePreference::Memory => {
                log::info!("Using an in-memory keystore as requested.");
                let cs: Arc<CredentialStore> =
                    keyring_core::sample::Store::new().context(KeystoreCreateSnafu)?;
                Ok(cs)
            }
            KeystorePreference::Default => match get_native_keystore() {
                Ok(Some(s)) => Ok(s),
                Ok(None) => {
                    log::warn!("No native keystore for the current platform.");
                    get_fallback_keystore()
                }
                Err(msg) => {
                    log::warn!("Error installing native keystore: {}", msg);
                    get_fallback_keystore()
                }
            },
        }
    }

    pub fn create(renku_url: RenkuUrl) -> Result<KeyringStore, Error> {
        Ok(KeyringStore {
            renku_url,
            store: Self::create_underlying_keyring()?,
        })
    }

    fn build_entry(&self) -> Result<keyring_core::Entry, Error> {
        let service = self.renku_url.as_url().domain().unwrap_or("renku");
        let user = whoami::account().unwrap_or_else(|_| "default-user".to_string());
        self.store
            .as_ref()
            .build(service, &user, None)
            .context(BuildEntrySnafu)
    }
}

impl Keystore for KeyringStore {
    fn write_token(&self, token: &Response) -> Result<(), Error> {
        let entry = self.build_entry()?;
        let cnt = serde_json::to_vec(token).context(ToJsonSnafu)?;
        entry.set_secret(&cnt).context(WriteSecretSnafu)?;
        Ok(())
    }

    fn read_token(&self) -> Result<Option<Response>, Error> {
        let entry = self.build_entry()?;
        match entry.get_secret() {
            Ok(secret) => {
                let resp = serde_json::from_slice::<Response>(&secret).context(FromJsonSnafu)?;
                Ok(Some(resp))
            }
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(err) => Err(Error::ReadSecret { source: err }),
        }
    }

    fn clear(&self) -> Result<(), Error> {
        let entry = self.build_entry()?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(err) => Err(Error::WriteSecret { source: err }),
        }
    }
}

#[cfg(target_os = "windows")]
fn get_native_keystore() -> Result<Option<Arc<keyring_core::CredentialStore>>, Error> {
    use std::sync::Arc;
    use windows_native_keyring_store::Store as WindowsStore;

    WindowsStore::new()
        .context(KeystoreCreateSnafu)
        .map(|s| Some(s as Arc<keyring_core::CredentialStore>))
}

#[cfg(target_os = "linux")]
fn get_native_keystore() -> Result<Option<Arc<keyring_core::CredentialStore>>, Error> {
    use linux_keyutils_keyring_store::Store as KernelStore;
    use zbus_secret_service_keyring_store::Store as ZBusStore;

    match ZBusStore::new().context(KeystoreCreateSnafu) {
        Ok(store) => {
            log::info!("Use DBus secret service as keystore.");
            Ok(Some(store))
        }
        Err(msg) => {
            log::debug!("Error creating dbus keystore: {}", msg);
            log::info!("DBus secret service not available. Use kernel keyring.");
            let s = KernelStore::new().context(KeystoreCreateSnafu)?;
            Ok(Some(s))
        }
    }
}

#[cfg(target_os = "freebsd")]
fn get_native_keystore() -> Result<Option<Arc<keyring_core::CredentialStore>>, Error> {
    use zbus_secret_service_keyring_store::Store as ZBusStore;

    let store = ZBusStore::new().context(KeystoreCreateSnafu)?;
    log::info!("Use DBus secret service as keystore.");
    Ok(Some(store))
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "freebsd")))]
fn get_native_keystore() -> Result<Option<Arc<keyring_core::CredentialStore>>, Error> {
    Ok(None)
}

fn get_fallback_keystore() -> Result<Arc<keyring_core::CredentialStore>, Error> {
    let db_store = get_db_keystore();
    if let Err(msg) = db_store {
        log::warn!("Error setting up file-db keystore: {}", msg);
        let cs: Arc<CredentialStore> =
            keyring_core::sample::Store::new().context(KeystoreCreateSnafu)?;
        Ok(cs)
    } else {
        db_store
    }
}

/// Creates a cross-platform sqlite backed keystore.
fn get_db_keystore() -> Result<Arc<keyring_core::CredentialStore>, Error> {
    let db_dir = match ProjectDirs::from("io.renku", "sdsc", "renku-cli") {
        Some(pp) => {
            let dir = pp.data_dir();
            dir.to_path_buf()
        }
        None => std::env::temp_dir().join("renku-cli"),
    };
    let keystore_file = db_dir.join("keystore.db");
    log::debug!("Creating keystore in {:?}", keystore_file);
    std::fs::create_dir_all(db_dir).context(WriteFileSnafu)?;
    let config = DbKeyStoreConfig {
        path: keystore_file.clone(),
        encryption_opts: None,
        allow_ambiguity: false,
        vfs: None,
        index_always: false,
    };
    let store = DbKeyStore::new(config).context(KeystoreCreateSnafu)?;

    // file has been created after store is initialized
    set_readonly(&keystore_file)?;

    Ok(store)
}

#[cfg(unix)]
fn set_readonly(file: &PathBuf) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file)
        .context(WriteFileSnafu)?
        .permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(file, perms).context(WriteFileSnafu)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_readonly(_file: &PathBuf) -> Result<(), Error> {
    Ok(())
}
