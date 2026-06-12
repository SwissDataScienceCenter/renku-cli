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
//!   println!("{:?}", client.version().await);
//! };
//! ```
//!
//! # Authentication
//!
//! TODO

pub mod auth;
pub mod data;
pub mod keystore;
pub mod proxy;

use crate::data::project_id::ProjectId;
use crate::data::renku_url::RenkuUrl;

use self::data::*;
use auth::{Response, UserCode};
use keystore::{KeyringStore, Keystore};
use openidconnect::OAuth2TokenResponse;
use regex::Regex;
use reqwest::{Certificate, ClientBuilder, IntoUrl, RequestBuilder, Url};
use serde::{Serialize, de::DeserializeOwned};
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn display_bad_response(em: &Option<ErrorResponse>, body: &String) -> String {
    match em {
        Some(s) => match &s.error {
            Some(em) => em.message.to_owned(),
            None => s.message.to_owned().unwrap_or(body.to_owned()),
        },
        None => body.to_owned(),
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Keystore error: {}", source))]
    Keystore { source: keystore::Error },

    #[snafu(display("An error was received from {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display(
        "Response not successful: {} - {}",
        status,
        display_bad_response(err_message, body)
    ))]
    BadResponse {
        status: reqwest::StatusCode,
        body: String,
        url: String,
        err_message: Option<ErrorResponse>,
    },

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
}

/// The renku http client.
///
/// This wraps a reqwest client with methods corresonding to renku api
/// endpoints.
pub struct Client {
    client: reqwest::Client,
    settings: Settings,
    access_token: Option<String>,
    keystore: KeyringStore,
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

        let keystore = keystore::KeyringStore::create(renku_url.clone()).context(KeystoreSnafu)?;

        let auth_data = access_token.or(keystore
            .read_token()
            .context(KeystoreSnafu)?
            .map(|r| r.response.access_token().secret().clone()));
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
            keystore,
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

    async fn run_request<R: DeserializeOwned>(
        &self,
        req: RequestBuilder,
        url: Url,
    ) -> Result<R, Error> {
        log::debug!("Run request: {}", url);
        let resp = req.send().await.context(HttpSnafu { url: url.clone() })?;

        let status = resp.status();
        let body = resp.text().await.context(DeserializeRespSnafu)?;
        log::debug!("Response: {} -> {}", url, body);
        if status.is_success() {
            serde_json::from_str::<R>(&body).context(DeserializeJsonSnafu)
        } else {
            let err_resp = serde_json::from_str::<ErrorResponse>(&body).ok();
            Err(Error::BadResponse {
                status,
                body,
                url: url.to_string(),
                err_message: err_resp,
            })
        }
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get<R: DeserializeOwned>(&self, path: &str) -> Result<R, Error> {
        let url = self.make_url(path)?;
        log::debug!("JSON GET: {}", url);
        let req = self.set_bearer_token(self.client.get(url.clone()));
        self.run_request(req, url).await
    }

    /// Runs a POST request to the given url.
    async fn json_post<I: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &I,
    ) -> Result<R, Error> {
        let url = self.make_url(path)?;
        let req = self
            .set_bearer_token(self.client.post(url.clone()))
            .json::<I>(body);
        self.run_request(req, url).await
    }

    /// Runs a GET request to the given url. When `debug` is true, the
    /// response is first decoded into utf8 chars and logged at debug
    /// level. Otherwise bytes are directly decoded from JSON into the
    /// expected structure.
    async fn json_get_option<R: DeserializeOwned>(&self, path: &str) -> Result<Option<R>, Error> {
        let url = self.make_url(path)?;
        let req = self.set_bearer_token(self.client.get(url.clone()));

        let result = self.run_request(req, url).await;
        match result {
            Err(Error::BadResponse {
                status,
                body: _,
                url: _,
                err_message: _,
            }) => {
                if status == reqwest::StatusCode::NOT_FOUND {
                    Ok(None)
                } else {
                    result
                }
            }
            _ => result,
        }
    }

    /// Queries Renku for its version
    pub async fn version(&self) -> Result<VersionInfo, Error> {
        let renku = self.json_get::<SimpleVersion>("/api/data/version").await?;
        let renku_url = self.base_url().clone();
        Ok(VersionInfo { renku, renku_url })
    }

    pub async fn get_project(&self, id: &ProjectId) -> Result<Option<ProjectDetails>, Error> {
        match id {
            ProjectId::NamespaceSlug { namespace, slug } => {
                self.get_project_by_slug(namespace, slug).await
            }
            ProjectId::Id(pid) => self.get_project_by_id(pid).await,

            ProjectId::FullUrl(url) => self.get_project_by_url(url.as_url().clone()).await,
        }
    }

    /// Get project details given the namespace and slug.
    pub async fn get_project_by_slug(
        &self,
        namespace: &str,
        slug: &str,
    ) -> Result<Option<ProjectDetails>, Error> {
        log::debug!("Get project by namespace/slug: {}/{}", namespace, slug);
        let path = format!("/api/data/namespaces/{}/projects/{}", namespace, slug);
        let details = self.json_get_option::<ProjectDetails>(&path).await?;
        Ok(details)
    }

    /// Get project details by project id.
    pub async fn get_project_by_id(&self, id: &str) -> Result<Option<ProjectDetails>, Error> {
        log::debug!("Get project by id: {}", id);
        let path = format!("/api/data/projects/{}", id);
        let details = self.json_get_option::<ProjectDetails>(&path).await?;
        Ok(details)
    }

    /// Get project details by a ui url. It supports two formats:
    /// - /p/<ulid>
    /// - /p/<namespace>/<slug>
    pub async fn get_project_by_url<U: IntoUrl>(
        &self,
        url: U,
    ) -> Result<Option<ProjectDetails>, Error> {
        let urlstr = url.as_str().to_string();
        let url = url.into_url().context(HttpSnafu { url: urlstr })?;
        let mut base = url.clone();
        base.set_path("");
        let base_url = RenkuUrl::new(base);

        log::debug!("Get project by url: {}", &url);
        // there are different urls identifying the project
        //   /v2/projects/<id> (ui)
        //   /v2/projects/<namespace>/<slug> (ui)
        // note the ui urls are currently not stable
        let project_path_regex = Regex::new(
            r"(?x)
            (?<uiproj>/p/)(?<uiid>[0-7][0-9A-HJKMNP-TV-Z]{25}) # /p/<id> (ui)
            |
            (?<uinamespace>/p/)(?<uins>[^/]+)/(?<uiname>.+) # /p/<namespace>/<slug> (ui)",
        )
        .unwrap();
        let captures = project_path_regex.captures(url.path()).unwrap();

        log::debug!("Create temporary client for {}", &base_url);
        let client = Client::new(
            base_url,
            self.settings.proxy.clone(),
            self.settings.trusted_certificate.clone(),
            self.settings.accept_invalid_certs,
            self.access_token.clone(),
        )?;
        if captures.name("uiproj").is_some() {
            let proj_id = captures.name("uiid").unwrap().as_str();
            client.get_project_by_id(proj_id).await
        } else if captures.name("uinamespace").is_some() {
            let namespace = captures.name("uins").unwrap().as_str();
            let proj_name = captures.name("uiname").unwrap().as_str();
            client.get_project_by_slug(namespace, proj_name).await
        } else {
            Err(Error::ProjectUrlParse {
                reason: format!("Url {} did not match project URL pattern", url.path()),
            })
        }
    }

    pub async fn get_namespace(
        &self,
        first_slug: &str,
        second_slug: Option<&str>,
    ) -> Result<Option<NamespaceDetails>, Error> {
        log::debug!(
            "Get namespace by slug1/slug2: {}/{:?}",
            first_slug,
            second_slug
        );
        let path = if let Some(second) = second_slug {
            format!("/api/data/namespaces/{}/{}", first_slug, second)
        } else {
            format!("/api/data/namespaces/{}", first_slug)
        };

        let details = self.json_get_option::<NamespaceDetails>(&path).await?;
        Ok(details)
    }

    pub async fn start_session(
        &self,
        req: SessionStartRequest,
    ) -> Result<SessionStartResponse, Error> {
        log::debug!("Starting session: {}", req);

        let path = "/api/data/sessions";
        let details = self
            .json_post::<SessionStartRequest, SessionStartResponse>(path, &req)
            .await?;
        Ok(details)
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<(), Error> {
        log::debug!("Stop session: {}", session_id);
        let path = format!("/api/data/sessions/{}", session_id);
        let url = self.make_url(&path)?;
        self.set_bearer_token(self.client.delete(url.clone()))
            .send()
            .await
            .context(HttpSnafu { url })?;
        Ok(())
    }

    pub async fn list_sessions(&self, mode: Option<SessionMode>) -> Result<SessionList, Error> {
        let url = self.make_url("/api/data/sessions")?;
        log::debug!(
            "List sessions: {}?session_mode={}",
            url,
            mode.as_ref().map_or("", |e| e.to_query_param())
        );
        let mut req = self.set_bearer_token(self.client.get(url.clone()));
        if let Some(m) = mode {
            req = req.query(&[("session_type", m.to_query_param())])
        }

        self.run_request::<Vec<SessionStartResponse>>(req, url)
            .await
            .map(SessionList)
    }

    pub async fn session_logs(&self, session_id: &str) -> Result<SessionLogs, Error> {
        let path = format!("/api/data/sessions/{}/logs", session_id);
        let result = self.json_get::<SessionLogs>(&path).await?;
        Ok(result)
    }

    pub async fn start_login_flow(&self) -> Result<UserCode, Error> {
        let c = auth::get_user_code(self.settings.base_url.clone()).await?;
        Ok(c)
    }

    pub async fn complete_login_flow(&self, code: UserCode) -> Result<Response, Error> {
        let r = auth::poll_tokens(code).await?;
        self.keystore.write_token(&r).context(KeystoreSnafu)?;
        Ok(r)
    }

    pub async fn list_launchers(&self) -> Result<Vec<SessionLauncher>, Error> {
        let path = "/api/data/session_launchers";
        let result = self.json_get::<Vec<SessionLauncher>>(path).await?;
        Ok(result)
    }
}
