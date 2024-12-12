//! A http client for renku
//!
//! Provides a http client to Renku based on reqwest.
//!
//! # Usage
//!
//! ```rust
//! use rnk::httpclient;
//! use rnk::data::renku_url::RenkuUrl;
//!
//! let client = httpclient::Client::new(
//!    RenkuUrl::parse("https://renkulab.io").unwrap(),
//!    httpclient::proxy::ProxySetting::System,
//!    None,
//!    false,
//!    None,
//! ).unwrap();
//! async {
//!   println!("{:?}", client.version(false).await);
//! };
//! ```
//!
//! # Authentication
//!
//! TODO

pub mod auth;
mod cache;
pub mod data;
pub mod proxy;

use crate::data::project_id::ProjectId;
use crate::data::renku_url::RenkuUrl;

use self::data::*;
use auth::{Response, UserCode};
use openidconnect::OAuth2TokenResponse;
use regex::Regex;
use reqwest::{Certificate, ClientBuilder, IntoUrl, RequestBuilder, Url};
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

    #[snafu(display("Error parsing url: {}", reason))]
    ProjectUrlParse { reason: String },
    #[snafu(transparent)]
    Auth { source: auth::AuthError },

    #[snafu(transparent)]
    Cache { source: cache::Error },
}

/// The renku http client.
///
/// This wraps a reqwest client with methods corresonding to renku api
/// endpoints.
pub struct Client {
    client: reqwest::Client,
    settings: Settings,
    access_token: Option<String>,
}

#[derive(Debug)]
struct Settings {
    proxy: proxy::ProxySetting,
    trusted_certificate: Option<PathBuf>,
    accept_invalid_certs: bool,
    base_url: RenkuUrl,
}

impl Client {
    pub fn new(
        renku_url: RenkuUrl,
        proxy: proxy::ProxySetting,
        trusted_certificate: Option<PathBuf>,
        accept_invalid_certs: bool,
        access_token: Option<String>,
    ) -> Result<Client, Error> {
        log::debug!("Create renku client for: {}", renku_url);
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

        let auth_data = access_token
            .or(cache::read_auth_token()?.map(|r| r.response.access_token().secret().clone()));
        let client = client_builder.build().context(ClientCreateSnafu)?;
        Ok(Client {
            client,
            access_token: auth_data,
            settings: Settings {
                proxy,
                trusted_certificate,
                accept_invalid_certs,
                base_url: renku_url,
            },
        })
    }

    pub fn base_url(&self) -> &RenkuUrl {
        &self.settings.base_url
    }

    fn make_url(&self, path: &str) -> Result<Url, Error> {
        self.settings
            .base_url
            .as_url()
            .join(path)
            .context(UrlParseSnafu)
    }

    fn set_bearer_token(&self, b: RequestBuilder) -> RequestBuilder {
        match &self.access_token {
            Some(token) => b.bearer_auth(token),
            None => b,
        }
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get<R: DeserializeOwned>(&self, path: &str, debug: bool) -> Result<R, Error> {
        let url = self.make_url(path)?;
        log::debug!("JSON GET: {}", url);
        let resp = self
            .set_bearer_token(self.client.get(url.clone()))
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
            .set_bearer_token(self.client.get(url.clone()))
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

            ProjectId::FullUrl(url) => self.get_project_by_url(url.as_url().clone(), debug).await,
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
        let path = format!("/api/data/namespaces/{}/projects/{}", namespace, slug);
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
        //   /v2/projects/<id> (ui)
        //   /v2/projects/<namespace>/<slug> (ui)
        // note the ui urls are currently not stable
        let project_path_regex = Regex::new(
            r"(?x)
            (?<uiproj>/v2/projects/)(?<uiid>[0-7][0-9A-HJKMNP-TV-Z]{25}) # /v2/projects/<id> (ui)
            |
            (?<uinamespace>/v2/projects/)(?<uins>[^/]+)/(?<uiname>.+) # /v2/projects/<namespace>/<slug> (ui)").unwrap();
        let captures = project_path_regex.captures(url.path()).unwrap();
        let path = if captures.name("uiproj").is_some() {
            &format!(
                "/api/data/projects/{}",
                captures.name("uiid").unwrap().as_str()
            )
        } else {
            return Err(Error::ProjectUrlParse {
                reason: format!("Url {} did not match project URL pattern", url),
            });
        };

        log::debug!("Transformed path {} to: {}", url.path(), &path);
        let mut base = url.clone();
        base.set_path("");
        let base_url = RenkuUrl::new(base);

        log::debug!("Create temporary client for {}", &base_url);
        let client = Client::new(
            base_url,
            self.settings.proxy.clone(),
            self.settings.trusted_certificate.clone(),
            self.settings.accept_invalid_certs,
            self.access_token.clone(),
        )?;

        let details = client
            .json_get_option::<ProjectDetails>(&path, debug)
            .await?;
        Ok(details)
    }

    pub async fn start_login_flow(&self) -> Result<UserCode, Error> {
        let c = auth::get_user_code(self.settings.base_url.clone()).await?;
        Ok(c)
    }

    pub async fn complete_login_flow(&self, code: UserCode) -> Result<Response, Error> {
        let r = auth::poll_tokens(code).await?;
        cache::write_auth_token(&r).await?;
        Ok(r)
    }
}
