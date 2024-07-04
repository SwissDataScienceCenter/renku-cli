//! Defines data structures for requests and responses and their
//! `De/Serialize` instances.

use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum Visibility {
    #[serde(alias = "public")]
    Public,
    #[serde(alias = "private")]
    Private,
}
impl Visibility {
    pub fn as_string(&self) -> &str {
        match self {
            Visibility::Private => "private",
            Visibility::Public => "public",
        }
    }
}
impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
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
