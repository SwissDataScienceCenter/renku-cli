//! Defines data structures for requests and responses and their
//! `De/Serialize` instances.

use serde::{Deserialize, Serialize};

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
