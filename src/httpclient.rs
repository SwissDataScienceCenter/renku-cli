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

use crate::util::data::ProjectId;

use self::data::*;
use reqwest::Certificate;
use reqwest::ClientBuilder;
use reqwest::IntoUrl;
use reqwest::Url;
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

    #[snafu(display("Error reading url: {}", source))]
    UrlParse { source: url::ParseError },
}

/// The renku http client.
///
/// This wraps a reqwest client with methods corresonding to renku api
/// endpoints.
pub struct Client {
    client: reqwest::Client,
    settings: Settings,
}

#[derive(Debug)]
struct Settings {
    proxy: proxy::ProxySetting,
    trusted_certificate: Option<PathBuf>,
    accept_invalid_certs: bool,
    base_url: Url,
}

impl Client {
    pub fn new<U: IntoUrl>(
        renku_url: U,
        proxy: proxy::ProxySetting,
        trusted_certificate: Option<PathBuf>,
        accept_invalid_certs: bool,
    ) -> Result<Client, Error> {
        let urlstr = renku_url.as_str().to_string();
        let url = renku_url.into_url().context(HttpSnafu { url: urlstr })?;
        log::debug!("Create renku client for: {}", url);
        let mut client_builder = ClientBuilder::new().user_agent(USER_AGENT);
        client_builder = proxy.set(client_builder).context(ClientCreateSnafu)?;
        match &trusted_certificate {
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
            settings: Settings {
                proxy,
                trusted_certificate,
                accept_invalid_certs,
                base_url: url,
            },
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.settings.base_url
    }

    fn make_url(&self, path: &str) -> Result<Url, Error> {
        self.settings.base_url.join(path).context(UrlParseSnafu)
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get<R: DeserializeOwned>(&self, path: &str, debug: bool) -> Result<R, Error> {
        let url = self.make_url(path)?;
        log::debug!("JSON GET: {}", url);
        let resp = self
            .client
            .get(url.clone())
            .send()
            .await
            .context(HttpSnafu { url: url.clone() })?;
        if debug {
            let body = resp.text().await.context(DeserializeRespSnafu)?;
            log::debug!("GET {} -> {}", url, body);
            serde_json::from_str::<R>(&body).context(DeserializeJsonSnafu)
        } else {
            resp.json::<R>().await.context(DeserializeRespSnafu)
        }
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get_option<R: DeserializeOwned>(
        &self,
        path: &str,
        debug: bool,
    ) -> Result<Option<R>, Error> {
        let url = self.make_url(path)?;
        let resp = self
            .client
            .get(url.clone())
            .send()
            .await
            .context(HttpSnafu { url: url.clone() })?;

        if debug {
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                log::debug!("GET {} -> NotFound", &url);
                Ok(None)
            } else {
                let body = &resp.text().await.context(DeserializeRespSnafu)?;
                log::debug!("GET {} -> {}", &url, body);
                let r = serde_json::from_str::<R>(body).context(DeserializeJsonSnafu)?;
                Ok(Some(r))
            }
        } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            let r = resp.json::<R>().await.context(DeserializeRespSnafu)?;
            Ok(Some(r))
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

    pub async fn get_project(
        &self,
        id: &ProjectId,
        debug: bool,
    ) -> Result<Option<ProjectDetails>, Error> {
        match id {
            ProjectId::NamespaceSlug { namespace, slug } => {
                self.get_project_by_slug(namespace, slug, debug).await
            }
            ProjectId::Id(pid) => self.get_project_by_id(pid, debug).await,

            ProjectId::FullUrl(url) => self.get_project_by_url(url.clone(), debug).await,
        }
    }

    /// Get project details given the namespace and slug.
    pub async fn get_project_by_slug(
        &self,
        namespace: &str,
        slug: &str,
        debug: bool,
    ) -> Result<Option<ProjectDetails>, Error> {
        log::debug!("Get project by namespace/slug: {}/{}", namespace, slug);
        let path = format!("/api/data/projects/{}/{}", namespace, slug);
        let details = self.json_get_option::<ProjectDetails>(&path, debug).await?;
        Ok(details)
    }

    /// Get project details by project id.
    pub async fn get_project_by_id(
        &self,
        id: &str,
        debug: bool,
    ) -> Result<Option<ProjectDetails>, Error> {
        log::debug!("Get project by id: {}", id);
        let path = format!("/api/data/projects/{}", id);
        let details = self.json_get_option::<ProjectDetails>(&path, debug).await?;
        Ok(details)
    }

    pub async fn get_project_by_url<U: IntoUrl>(
        &self,
        url: U,
        debug: bool,
    ) -> Result<Option<ProjectDetails>, Error> {
        let urlstr = url.as_str().to_string();
        let url = url.into_url().context(HttpSnafu { url: urlstr })?;
        log::debug!("Get project by url: {}", &url);
        // there are different urls identifying the project
        //   /api/data/projects/<id>
        //   /api/data/projects/<namespace>/<slug>
        //   /v2/projects/<id> (ui)
        //   /v2/projects/<namespace>/<slug> (ui)
        // the api is only the first two. Try to replace `v2` with `api/data`
        // note the ui urls are currently not stable

        let path = match url.path_segments() {
            Some(it) => {
                let mut seen = false;
                it.flat_map(|s| {
                    if s == "v2" && !seen {
                        seen = true;
                        vec!["api", "data"]
                    } else {
                        vec![s]
                    }
                })
                .fold(String::new(), |a, b| a + b + "/")
            }
            None => url.path().to_string(),
        };

        log::debug!("Transformed path {} to: {}", url.path(), &path);
        let mut base = url.clone();
        base.set_path("");
        let base_url = base.to_string();

        log::debug!("Create temporary client for {}", &base_url);
        let client = Client::new(
            base_url,
            self.settings.proxy.clone(),
            self.settings.trusted_certificate.clone(),
            self.settings.accept_invalid_certs,
        )?;

        let details = client
            .json_get_option::<ProjectDetails>(&path, debug)
            .await?;
        Ok(details)
    }
}
