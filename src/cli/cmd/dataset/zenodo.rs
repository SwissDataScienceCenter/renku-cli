use crate::httpclient::{Error as HttpError, HttpSnafu, UrlParseSnafu};
use reqwest;
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;
use walkdir::WalkDir;

pub struct ZenodoClient {
    http_client: reqwest::Client,
    base_url: Url,
    token: String,
}

const BASE_URL: Url = Url::parse("https://zenodo.org").unwrap();

#[derive(Debug, Serialize, Deserialize)]
pub struct Deposition {
    pub search: SearchServiceVersion,
    pub data: SimpleVersion,
}

impl ZenodoClient {
    pub fn new(token: String) -> ZenodoClient {
        let http_client = reqwest::Client::new();
        return ZenodoClient {
            http_client,
            base_url: BASE_URL,
            token,
        };
    }

    fn make_url(&self, path: &str) -> Result<Url, Error> {
        return self.base_url.join(path).context(UrlParseSnafu);
    }

    pub async fn get_deposition<R: DeserializeOwned>(&self, id: &str) -> Result<R, Error> {
        let endpoint = self.make_url(format!("/api/deposit/depositions/{id}"));
        let res = self
            .http_client
            .get(enpdoint)
            .send()
            .await
            .context(HttpSnafu { url: endpoint })?;
        res.error_for_status()?;
        return res.json();
    }

    pub async fn upload_files(&self, deposition_id: &str, source_path: &Path) -> Result<(), Error> {
        for f in WalkDir::new(source_path) {
            self.upload_file(deposition_id, f?.path()).await?;
        }
    }

    pub async fn list_files<R: DeserializeOwned>(&self, deposition_id: &str) -> Result<R, Error> {
        let endpoint = self.make_url(format!("/api/deposit/depositions/{id}/files"));
        let res = self
            .http_client
            .get(enpdoint)
            .send()
            .await
            .context(HttpSnafu { url: endpoint })?;
        res.error_for_status()?;
        return res.json();
    }

    async fn upload_file<R: DeserializeOwned>(
        &self,
        deposition_id: &str,
        file_path: &Path,
    ) -> Result<R, Error> {
        let file = File::open(file_path).await?;
        let endpoint = self.make_url(format!("/api/deposit/depositions/{id}/files"));
        let stream = FramedRead::new(file, BytesCodec::new());
        let form = reqwest::multipart::Form::new()
            .text("name", file_path.file_name()?.to_str()?)
            .part("file", stream);
        let res = self
            .http_client
            .post(endpoint)
            .multipart(form)
            .send()
            .await
            .context(HttpSnafu { url: endpoint })?;
        res.error_for_status()?;
        return res.json();
    }
}
