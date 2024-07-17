use std::fmt;
use std::str;

use serde::Serialize;
use snafu::{ResultExt, Snafu};
use url::{ParseError as UrlParseError, Url};

#[derive(Debug)]
pub enum ProjectId {
    NamespaceSlug { namespace: String, slug: String },
    Id(String),
    FullUrl(Url),
}

impl ProjectId {
    pub fn as_string(&self) -> String {
        match self {
            ProjectId::NamespaceSlug { namespace, slug } => {
                format!("{}/{}", namespace, slug)
            }
            ProjectId::Id(id) => id.to_string(),
            ProjectId::FullUrl(url) => url.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Snafu)]
pub enum ProjectIdParseError {
    UrlParse { source: UrlParseError },
}

impl str::FromStr for ProjectId {
    type Err = ProjectIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http") {
            let u = Url::parse(s).context(UrlParseSnafu)?;
            Ok(ProjectId::FullUrl(u))
        } else {
            match s.split_once('/') {
                Some((pre, suf)) => Ok(ProjectId::NamespaceSlug {
                    namespace: pre.into(),
                    slug: suf.into(),
                }),
                None => Ok(ProjectId::Id(s.to_string())),
            }
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
