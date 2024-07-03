//! A http client for renku
//!
//! Provides a http client to Renku based on reqwest.
//!
//! # Usage
//!
//! ```rust
//! use rnk::httpclient;
//! let client = httpclient::Client::new(
//!    "https://renkulab.io",
//!    httpclient::proxy::ProxySetting::System,
//!    &None,
//!    false
//! ).unwrap();
//! async {
//!   println!("{:?}", client.version(false).await);
//! };
//! ```
//!
//! # Authentication
//!
//! TODO

pub mod data;
pub mod proxy;

use self::data::*;
use reqwest::Certificate;
use reqwest::ClientBuilder;
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An error was received from {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("An error occurred creating the http client: {}", source))]
    ClientCreate { source: reqwest::Error },

    #[snafu(display("Error opening file '{}': {}", path.display(), source))]
    OpenFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("An error occured reading the response: {}", source))]
    DeserializeResp { source: reqwest::Error },

    #[snafu(display("An error occured reading the response: {}", source))]
    DeserializeJson { source: serde_json::Error },
}

/// The renku http client.
///
/// This wraps a reqwest client with methods corresonding to renku api
/// endpoints.
pub struct Client {
    client: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new<S: Into<String>>(
        renku_url: S,
        proxy: proxy::ProxySetting,
        trusted_certificate: &Option<PathBuf>,
        accept_invalid_certs: bool,
    ) -> Result<Client, Error> {
        let url = renku_url.into();
        log::debug!("Create renku client for: {}", url);
        let mut client_builder = ClientBuilder::new().user_agent(USER_AGENT);
        client_builder = proxy.set(client_builder).context(ClientCreateSnafu)?;
        match trusted_certificate {
            Some(cert_file) => {
                log::debug!(
                    "Adding extra certificate from file: {}",
                    cert_file.display(),
                );
                let buf = std::fs::read(cert_file).context(OpenFileSnafu { path: cert_file })?;
                let cert = match Certificate::from_pem(&buf) {
                    Ok(c) => c,
                    Err(e) => {
                        log::debug!("Reading PEM format failed: {:?}. Try with DER", e);
                        Certificate::from_der(&buf).context(ClientCreateSnafu)?
                    }
                };
                client_builder = client_builder.add_root_certificate(cert);
            }
            None => {
                if accept_invalid_certs {
                    log::info!("NOTE: ignoring invalid certificates!");
                    client_builder = client_builder.danger_accept_invalid_certs(true);
                }
            }
        }

        let client = client_builder.build().context(ClientCreateSnafu)?;
        Ok(Client {
            client,
            base_url: url,
        })
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get<R: DeserializeOwned>(&self, path: &str, debug: bool) -> Result<R, Error> {
        let url = &format!("{}{}", self.base_url, path);
        if debug {
            let resp = self
                .client
                .get(url)
                .send()
                .await
                .context(HttpSnafu { url })?
                .text()
                .await
                .context(DeserializeRespSnafu)?;
            log::debug!("GET {} -> {}", url, resp);
            serde_json::from_str::<R>(&resp).context(DeserializeJsonSnafu)
        } else {
            self.client
                .get(url)
                .send()
                .await
                .context(HttpSnafu { url })?
                .json::<R>()
                .await
                .context(DeserializeRespSnafu)
        }
    }

    /// Queries Renku for its version
    pub async fn version(&self, debug: bool) -> Result<VersionInfo, Error> {
        let data = self
            .json_get::<SimpleVersion>("/ui-server/api/data/version", debug)
            .await?;
        let search = self
            .json_get::<SearchServiceVersion>("/ui-server/api/search/version", debug)
            .await?;
        Ok(VersionInfo { search, data })
    }
}
