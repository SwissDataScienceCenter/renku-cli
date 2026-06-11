//! Defines data structures for requests and responses and their
//! `De/Serialize` instances.

use crate::data::{renku_url::RenkuUrl, submission_id::SubmissionId};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap, fmt};
use tabled::{
    Table,
    builder::Builder,
    settings::{Settings, Style},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionLauncher {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub launcher_type: SessionMode,
}
impl fmt::Display for SessionLauncher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionLogs(pub HashMap<String, String>);

impl fmt::Display for SessionLogs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.0 {
            writeln!(f, "- {}", k)?;
            write!(f, "{}", v)?;
        }
        write!(f, "")
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SessionMode {
    #[serde(rename = "interactive")]
    Interactive,
    #[serde(rename = "non-interactive")]
    NonInteractive,
}

impl fmt::Display for SessionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_query_param())
    }
}

impl SessionMode {
    pub fn to_query_param(&self) -> &str {
        match self {
            SessionMode::Interactive => "interactive",
            SessionMode::NonInteractive => "non-interactive",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStartRequest {
    pub launcher_id: String,
    pub session_type: String,
    pub submission_id: Option<SubmissionId>,
    pub job_args_override: Option<Vec<String>>,
    pub job_command_override: Option<Vec<String>>,
}
impl fmt::Display for SessionStartRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SessionStart(launcher={}, session_type={}, submission_id={:?}, job_args_overrides={:?}, command={:?})",
            self.launcher_id,
            self.session_type,
            self.submission_id,
            self.job_args_override,
            self.job_command_override
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionList(pub Vec<SessionStartResponse>);

fn create_session_table<I, T>(data: I) -> Table
where
    I: IntoIterator<Item = T>,
    T: Borrow<SessionStartResponse>,
{
    let mut builder = Builder::default();
    for e in data {
        let r = e.borrow();
        let sub_id = match &r.submission_id {
            Some(n) => n,
            None => "-",
        };
        let started = r.started.format();
        let data = vec![&r.name, sub_id, &r.project_id, &r.status.state, &started];
        builder.push_record(data);
    }
    builder.insert_record(
        0,
        vec!["Job", "Submission Id", "Project Id", "Status", "Started"],
    );

    let mut table = builder.build();
    let settings = Settings::default().with(Style::sharp());

    table.with(settings);
    table
}

impl fmt::Display for SessionList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "No jobs found.")
        } else {
            let table = create_session_table(&self.0);
            write!(f, "{}", table)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatus {
    message: Option<String>,
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStartResponse {
    image: String,
    name: String,
    project_id: String,
    launcher_id: String,
    submission_id: Option<String>,
    status: SessionStatus,
    started: Timestamp,
}

impl fmt::Display for SessionStartResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table = create_session_table(vec![self]);
        write!(f, "{}", table)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Visibility {
    #[serde(alias = "public")]
    Public,
    #[serde(alias = "private")]
    Private,
}
impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Visibility::Private => "private",
                Visibility::Public => "public",
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchServiceVersion {
    pub name: String,
    pub version: String,
    #[serde(alias = "headCommit")]
    pub head_commit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleVersion {
    pub version: String,
}

/// Describes the version information provided by the renku platform.
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub renku_url: RenkuUrl,
    pub renku: SimpleVersion,
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Renku Platform:\nUrl: {}\nVersion: {}",
            self.renku_url, self.renku.version
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamespaceDetails {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub path: String,
}
impl fmt::Display for NamespaceDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Namespace: {} ({})", self.path, self.id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDetails {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub slug: String,
    pub visibility: Visibility,
    pub etag: Option<String>,
    pub repositories: Vec<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub creation_date: Timestamp,
}
impl fmt::Display for ProjectDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .repositories
            .iter()
            .fold(String::new(), |a, b| a + "\n  - " + b);
        write!(
            f,
            "Id: {}\nNamespace/Slug: {}/{}\nVisibility: {}\nCreated At: {}\nRepositories:{}",
            self.id, self.namespace, self.slug, self.visibility, self.creation_date, lines
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenkuError {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for RenkuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {} - {}", self.code, self.message)
    }
}

/// Error response can be either a concrete renku error, or an error
/// from the proxy/gateway then there is only a message field.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: Option<RenkuError>,
    pub message: Option<String>,
}

impl ErrorResponse {
    pub fn code(&self) -> Option<i32> {
        self.error.as_ref().map(|em| em.code)
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            Some(re) => write!(f, "{}", re),
            None => {
                if let Some(m) = &self.message {
                    write!(f, "{}", m)
                } else {
                    write!(f, "No error message.")
                }
            }
        }
    }
}
