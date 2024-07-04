use std::fmt;
use std::str;

use serde::Serialize;
use snafu::Snafu;

#[derive(Debug)]
pub enum ProjectId {
    NamespaceSlug { namespace: String, slug: String },
    Id(String),
}

impl ProjectId {
    pub fn as_string(&self) -> String {
        match self {
            ProjectId::NamespaceSlug { namespace, slug } => {
                format!("{}/{}", namespace, slug)
            }
            ProjectId::Id(id) => id.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Snafu)]
pub struct ProjectIdParseError;

impl str::FromStr for ProjectId {
    type Err = ProjectIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((pre, suf)) => Ok(ProjectId::NamespaceSlug {
                namespace: pre.into(),
                slug: suf.into(),
            }),
            None => Ok(ProjectId::Id(s.to_string())),
        }
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Debug, Serialize)]
pub struct SimpleMessage {
    pub message: String,
}

impl fmt::Display for SimpleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
