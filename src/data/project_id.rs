use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use url::ParseError as UrlParseError;

use super::renku_url::RenkuUrl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectId {
    NamespaceSlug { namespace: String, slug: String },
    Id(String),
    FullUrl(RenkuUrl),
}

#[derive(Debug, PartialEq, Snafu)]
pub enum ProjectIdParseError {
    UrlParse { source: UrlParseError },
}

impl ProjectId {
    pub fn parse(s: &str) -> Result<ProjectId, ProjectIdParseError> {
        s.parse::<ProjectId>()
    }
}

impl FromStr for ProjectId {
    type Err = ProjectIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http") {
            let u = RenkuUrl::parse(s).context(UrlParseSnafu)?;
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
        write!(
            f,
            "{}",
            match self {
                ProjectId::NamespaceSlug { namespace, slug } => {
                    format!("{}/{}", namespace, slug)
                }
                ProjectId::Id(id) => id.to_string(),
                ProjectId::FullUrl(url) => url.to_string(),
            }
        )
    }
}

#[test]
fn read_to_string() {
    let id1 = ProjectId::NamespaceSlug {
        namespace: "n1".into(),
        slug: "s1".into(),
    };
    let id2 = ProjectId::Id("pr-id-42".into());
    let id3 = ProjectId::FullUrl(RenkuUrl::parse("http://localhost/project/1").unwrap());

    for id in vec![id1, id2, id3] {
        let id_str = format!("{}", id);
        let id_parsed = ProjectId::parse(&id_str).unwrap();
        assert_eq!(id, id_parsed);
    }
}
