use futures::stream::StreamExt;
use md5::Context as Md5Context;
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{self, Body, Response, StatusCode};
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};
use std::path::{Path, PathBuf, StripPrefixError};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::io::ReaderStream;
use url::{ParseError, Url};
use walkdir::{DirEntry, WalkDir};

use crate::cli::cmd::dataset::zenodo_api::{FileResponse, FileUploadResponse};

use super::zenodo_api::DepositionResponse;
use std::io;
use std::sync::LazyLock;

pub struct ZenodoClient {
    http_client: reqwest::Client,
    base_url: Url,
    token: String,
    debug: bool,
}

static BASE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://zenodo.org").expect("Invalid Base URL config"));

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A zendoo client error occured: {}", source))]
    Reqwest { source: reqwest::Error },

    #[snafu(display("A directory listing error occured: {}", source))]
    DirWalk { source: walkdir::Error },

    #[snafu(display("An error occured reading the response: {}", source))]
    DeserializeResp { source: reqwest::Error },

    #[snafu(display("An error occured reading the file: {}", source))]
    FileReading { source: io::Error },

    #[snafu(display("An error occured parsing the file path: {}", fp.to_string_lossy()))]
    FileParsing { fp: PathBuf },

    #[snafu(display("An error occured parsing the url {}", source))]
    UrlParse { source: ParseError },

    #[snafu(display("Stripping file prefix failed {}", source))]
    StripPathPrefix { source: StripPrefixError },

    #[snafu(display("An error occured desearializing the response to json: {}", source))]
    DeserializeJson { source: serde_json::Error },

    #[snafu(display(
        "The request to zenodo at {} resulted in an unexpected status code {}",
        url,
        status_code,
    ))]
    FailedRequest { url: Url, status_code: StatusCode },

    #[snafu(display("Directories like {} cannot be uploaded in Zenodo, please zip the directory and then upload the archive.", fp.display()))]
    DirUpload { fp: PathBuf },
}

impl ZenodoClient {
    pub fn new(token: String, debug: bool) -> ZenodoClient {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        let http_client = reqwest::Client::builder()
            .user_agent("SDSC/renku") // Zenodo rejects the reqwest User Agent
            .build()
            .unwrap();
        ZenodoClient {
            http_client,
            base_url: BASE_URL.clone(),
            token,
            debug,
        }
    }

    fn make_url(&self, path: &str) -> Result<Url, Error> {
        self.base_url.join(path).context(UrlParseSnafu)
    }

    pub async fn get_depositions(&self) -> Result<Vec<DepositionResponse>, Error> {
        let endpoint = self.make_url("/api/deposit/depositions")?;
        let res = self
            .http_client
            .get(endpoint)
            .bearer_auth(&self.token)
            .send()
            .await
            .context(ReqwestSnafu)?;
        Self::json_parse(res, self.debug).await
    }

    pub async fn get_deposition(&self, deposition_id: &str) -> Result<DepositionResponse, Error> {
        let endpoint = self.make_url(&format!("/api/deposit/depositions/{deposition_id}"))?;
        let res = self
            .http_client
            .get(endpoint)
            .bearer_auth(&self.token)
            .send()
            .await
            .context(ReqwestSnafu)?;
        Self::json_parse(res, self.debug).await
    }

    pub async fn upload_files(&self, deposition_id: &str, source_path: &Path) -> Result<(), Error> {
        let dep = self.get_deposition(deposition_id).await?;
        if dep.links.bucket.is_none() {
            println!("Could not find the expected bucket link in the deposit");
            return Ok(());
        }
        let bucket = &dep.links.bucket.unwrap();
        fn is_hidden(entry: &DirEntry) -> bool {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        }
        let walker = WalkDir::new(source_path).into_iter();
        for (f_ind, f) in walker.filter_entry(|e| !is_hidden(e)).enumerate() {
            let path = f.context(DirWalkSnafu)?;
            let path_std = path.path();
            if f_ind == 0 && path_std.is_dir() {
                // If the source path is a dir the first entry of the walk is the same dir that was
                // passed
                continue;
            }
            if path_std.is_dir() {
                // If a directory is encountered we fail because zenodo does not support
                // uploading directories
                return Err(Error::DirUpload {
                    fp: path_std.to_path_buf(),
                });
            }
            let remote_path = path_std
                .file_name()
                .ok_or(Error::FileParsing {
                    fp: path_std.to_path_buf(),
                })?
                .to_str()
                .ok_or(Error::FileParsing {
                    fp: path_std.to_path_buf(),
                })?;
            log::info!("uploading file {} -> {}", path_std.display(), remote_path);
            let existing_files = self.list_files(deposition_id).await?;
            self.upload_file(bucket, path_std, remote_path, existing_files)
                .await?;
        }
        Ok(())
    }

    pub async fn list_files(&self, deposition_id: &str) -> Result<Vec<FileResponse>, Error> {
        let endpoint = self.make_url(&format!("/api/deposit/depositions/{deposition_id}/files"))?;
        let res = self
            .http_client
            .get(endpoint)
            .bearer_auth(&self.token)
            .send()
            .await
            .context(ReqwestSnafu)?;
        Self::json_parse(res, self.debug).await
    }

    async fn upload_file(
        &self,
        bucket_url: &str,
        local_file: &Path,
        remote_file: &str,
        existing_files: Vec<FileResponse>,
    ) -> Result<FileUploadResponse, Error> {
        let file = File::open(local_file).await.context(FileReadingSnafu)?;
        let metadata = file.metadata().await.context(FileReadingSnafu)?;
        let file_size = metadata.len();
        let file_may_exist = existing_files
            .iter()
            .find(|f| f.filename == remote_file && f.filesize == file_size as f64);
        if let Some(existing_file) = file_may_exist {
            let hash = Self::md5_hash(local_file).await?;
            if hash == file_may_exist.unwrap().checksum {
                log::info!(
                    "File {} exists already and hash matches, skipping.",
                    remote_file
                );
                return Ok(FileUploadResponse {
                    key: existing_file.filename.to_owned(),
                    mimetype: "".to_owned(),
                    checksum: existing_file.checksum.to_owned(),
                    size: existing_file.filesize as u64,
                });
            }
        }
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);
        let res = self
            .http_client
            .put(format!(
                "{}/{}",
                bucket_url.strip_suffix("/").unwrap_or(bucket_url),
                remote_file.strip_prefix("/").unwrap_or(remote_file)
            ))
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, file_size)
            .bearer_auth(&self.token)
            .body(file_body)
            .send()
            .await
            .context(ReqwestSnafu)?;
        Self::json_parse(res, self.debug).await
    }

    async fn json_parse<R: DeserializeOwned>(resp: Response, debug: bool) -> Result<R, Error> {
        let url = resp.url().to_owned();
        let status = resp.status();
        if !resp.status().is_success() {
            let body = resp.text().await.context(DeserializeRespSnafu).unwrap();
            log::debug!(
                "The request at {} failed with status code {}, body {}",
                url,
                status,
                body,
            );
            return Err(Error::FailedRequest {
                url: url.to_owned(),
                status_code: status,
            });
        }
        if debug {
            let body = resp.text().await.context(DeserializeRespSnafu).unwrap();
            log::debug!(
                "Zenodo client request at {} responded with code {} and body {}",
                url,
                status,
                body
            );
            serde_json::from_str::<R>(&body).context(DeserializeJsonSnafu)
        } else {
            resp.json::<R>().await.context(DeserializeRespSnafu)
        }
    }

    async fn md5_hash(file_path: &Path) -> Result<String, Error> {
        let file = File::open(file_path).await.context(FileReadingSnafu)?;
        let mut reader = ReaderStream::new(file);
        let mut hasher = Md5Context::new();

        while let Some(chunk) = reader.next().await {
            let chunk = chunk.context(FileReadingSnafu)?; // Handle IO error
            hasher.consume(&chunk);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}
