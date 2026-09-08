use std::fmt::{self, Display};
use std::str::FromStr;

use crate::httpclient::{Client, Error};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq)]
pub enum LauncherIdOrName {
    Id(Ulid),
    Name(String),
}

#[derive(Debug)]
pub enum LauncherIdError {
    InvalidInput(String),
    RequestError(Error),
    NotFound(String),
}

impl LauncherIdOrName {
    pub fn parse(s: &str) -> Result<LauncherIdOrName, LauncherIdError> {
        s.parse::<LauncherIdOrName>()
    }
    pub async fn resolve(&self, client: &Client) -> Result<Ulid, LauncherIdError> {
        match self {
            LauncherIdOrName::Id(ulid) => Ok(*ulid),
            LauncherIdOrName::Name(name) => {
                let launchers = client
                    .list_launchers()
                    .await
                    .map_err(LauncherIdError::RequestError)?;
                match launchers.iter().find(|l| l.name == *name) {
                    Some(l) => {
                        l.id.parse::<Ulid>()
                            .map_err(|_| LauncherIdError::InvalidInput(l.id.clone()))
                    }
                    None => Err(LauncherIdError::NotFound(name.clone())),
                }
            }
        }
    }
}

impl FromStr for LauncherIdOrName {
    type Err = LauncherIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Ulid>()
            .map(LauncherIdOrName::Id)
            .unwrap_or_else(|_| LauncherIdOrName::Name(s.to_string())))
    }
}

impl fmt::Display for LauncherIdOrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LauncherIdOrName::Name(name) => {
                    name.to_string()
                }
                LauncherIdOrName::Id(id) => id.to_string(),
            }
        )
    }
}
impl Display for LauncherIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LauncherIdError::InvalidInput(msg) => write!(f, "Invalid launcher id: {}", msg),
            LauncherIdError::NotFound(name) => write!(f, "Launcher not found: {}", name),
            LauncherIdError::RequestError(error) => {
                write!(f, "Couldn't resolve launcher name: {}", error)
            }
        }
    }
}
impl std::error::Error for LauncherIdError {}

#[test]
fn read_to_string() {
    let id1 = LauncherIdOrName::Name("my-launcher".to_string());
    let id2 = LauncherIdOrName::Id(Ulid::generate());

    for id in [id1, id2] {
        let id_str = format!("{}", id);
        let id_parsed = LauncherIdOrName::parse(&id_str).unwrap();
        assert_eq!(id, id_parsed);
    }
}
