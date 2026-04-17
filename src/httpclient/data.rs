//! Defines data structures for requests and responses and their
//! `De/Serialize` instances.

use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionLogs(pub HashMap<String, String>);

impl fmt::Display for SessionLogs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.0 {
            write!(f, "- {}\n", k)?;
            write!(f, "{}", v)?;
        }
        write!(f, "")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionMode {
    Interactive,
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
}
impl fmt::Display for SessionStartRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SessionStart(launcher={}, session_type={})",
            self.launcher_id, self.session_type
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionList(pub Vec<SessionStartResponse>);

impl fmt::Display for SessionList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.0.iter().fold(String::new(), |a, b| {
            format!("{}\n - {} (image={})", a, b.name, b.image)
        });

        if self.0.is_empty() {
            write!(f, "No sessions found.")
        } else {
            write!(f, "Sessions:{}", lines)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStartResponse {
    image: String,
    name: String,
    project_id: String,
    launcher_id: String,
}

impl fmt::Display for SessionStartResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SessionStartResponse({}, image={})",
            self.name, self.image
        )
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
    pub search: SearchServiceVersion,
    pub data: SimpleVersion,
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
        match &self.error {
            Some(em) => Some(em.code),
            None => None,
        }
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
