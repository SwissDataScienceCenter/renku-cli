use std::path::PathBuf;

use directories::ProjectDirs;
use snafu::{ResultExt, Snafu};

use super::auth::Response;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Cannot write file: {}", source))]
    WriteFile { source: std::io::Error },

    #[snafu(display("Cannot read file: {}", source))]
    ReadFile { source: std::io::Error },

    #[snafu(display("Failed to create JSON contents: {}", source))]
    ToJson { source: serde_json::Error },

    #[snafu(display("Error decoding user code data: {}", source))]
    JsonDecode { source: serde_json::Error },
}

fn find_cache_dir() -> PathBuf {
    if let Some(pp) = ProjectDirs::from("io.renku", "sdsc", "renku-cli") {
        let dir = pp.data_dir();
        dir.to_path_buf()
    } else {
        std::env::temp_dir().join("renku-cli")
    }
}

fn token_file() -> PathBuf {
    find_cache_dir().join("token.json")
}

async fn token_file_create() -> Result<PathBuf, Error> {
    let dir = find_cache_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .context(WriteFileSnafu)?;
    let f = dir.join("token.json");
    Ok(f)
}

pub async fn write_auth_token(resp: &Response) -> Result<(), Error> {
    let file = token_file_create().await?;
    let cnt = serde_json::to_vec(resp).context(ToJsonSnafu)?;
    tokio::fs::write(&file, &cnt)
        .await
        .context(WriteFileSnafu)?;
    set_read_only(&file)
}

#[cfg(unix)]
fn set_read_only(file: &PathBuf) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file)
        .context(WriteFileSnafu)?
        .permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(file, perms).context(WriteFileSnafu)
}

#[cfg(not(unix))]
fn set_read_only(file: &PathBuf) -> Result<(), Error> {
    Ok(())
}

#[allow(dead_code)]
pub async fn read_auth_token_async() -> Result<Option<Response>, Error> {
    let file = token_file_create().await?;
    if file.try_exists().context(ReadFileSnafu)? {
        let buf = tokio::fs::read(file).await.context(ReadFileSnafu)?;
        let resp = serde_json::from_slice::<Response>(&buf).context(JsonDecodeSnafu)?;
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

pub fn read_auth_token() -> Result<Option<Response>, Error> {
    let file = token_file();
    if file.try_exists().context(ReadFileSnafu)? {
        let buf = std::fs::read(file).context(ReadFileSnafu)?;
        let resp = serde_json::from_slice::<Response>(&buf).context(JsonDecodeSnafu)?;
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}
