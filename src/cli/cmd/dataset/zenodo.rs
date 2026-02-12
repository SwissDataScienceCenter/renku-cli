use mime_guess::from_path;
use reqwest;
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::{ParseError, Url};
use walkdir::WalkDir;

use std::io;
use std::sync::LazyLock;
pub struct ZenodoClient {
    http_client: reqwest::Client,
    base_url: Url,
    token: String,
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
    #[snafu(display("An error occured reading the response: {}", source))]
    FileReading { source: io::Error },
    #[snafu(display("An error occured parsing the file path: {}", fp.to_string_lossy()))]
    FileParsing { fp: PathBuf },
    #[snafu(display("An error occured parsing the url {}", source))]
    UrlParse { source: ParseError },
}

impl ZenodoClient {
    pub fn new(token: String) -> ZenodoClient {
        let http_client = reqwest::Client::new();
        return ZenodoClient {
            http_client,
            base_url: BASE_URL.clone(),
            token,
        };
    }

    fn make_url(&self, path: &str) -> Result<Url, Error> {
        return self.base_url.join(path).context(UrlParseSnafu);
    }

    pub async fn get_deposition<R: DeserializeOwned>(
        &self,
        deposition_id: &str,
    ) -> Result<R, Error> {
        let endpoint = self.make_url(&format!("/api/deposit/depositions/{deposition_id}"))?;
        let res = self
            .http_client
            .get(endpoint)
            .send()
            .await
            .context(ReqwestSnafu)?;
        // res.error_for_status().context(ReqwestSnafu)?;
        return res.json::<R>().await.context(DeserializeRespSnafu);
    }

    pub async fn upload_files(&self, deposition_id: &str, source_path: &Path) -> Result<(), Error> {
        for f in WalkDir::new(source_path) {
            self.upload_file::<()>(deposition_id, f.context(DirWalkSnafu)?.path())
                .await?;
        }
        Ok(())
    }

    pub async fn list_files<R: DeserializeOwned>(&self, deposition_id: &str) -> Result<R, Error> {
        let endpoint = self.make_url(&format!("/api/deposit/depositions/{deposition_id}/files"))?;
        let res = self
            .http_client
            .get(endpoint)
            .send()
            .await
            .context(ReqwestSnafu)?;
        // res.error_for_status().context(ReqwestSnafu)?;
        return res.json::<R>().await.context(DeserializeRespSnafu);
    }

    async fn upload_file<R: DeserializeOwned>(
        &self,
        deposition_id: &str,
        file_path: &Path,
    ) -> Result<R, Error> {
        let file = File::open(file_path).await.context(FileReadingSnafu)?;
        let endpoint = self.make_url(&format!("/api/deposit/depositions/{deposition_id}/files"))?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_name = file_path
            .file_name()
            .ok_or(Error::FileParsing {
                fp: file_path.to_path_buf(),
            })?
            .to_str()
            .ok_or(Error::FileParsing {
                fp: file_path.to_path_buf(),
            })?;
        let mime_type = from_path(file_path).first_or_octet_stream();
        let form = reqwest::multipart::Form::new()
            .text("name", file_name.to_owned())
            .part(
                "file",
                reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(stream))
                    .file_name(file_name.to_owned())
                    .mime_str(mime_type.as_ref())
                    .context(ReqwestSnafu)?,
            );
        let res = self
            .http_client
            .post(endpoint)
            .multipart(form)
            .send()
            .await
            .context(ReqwestSnafu)?;
        // res.error_for_status().context(ReqwestSnafu)?;
        return res.json::<R>().await.context(DeserializeRespSnafu);
    }
}
