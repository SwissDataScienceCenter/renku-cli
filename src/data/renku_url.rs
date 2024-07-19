use std::{fmt::Display, str::FromStr};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use url::ParseError;

// Need this newtype to be able to implement Serialize and Deserialize

#[derive(Debug, PartialEq, Clone)]
pub struct RenkuUrl(Url);

impl RenkuUrl {
    pub fn new(url: Url) -> RenkuUrl {
        RenkuUrl(url)
    }

    pub fn parse(s: &str) -> Result<RenkuUrl, ParseError> {
        s.parse::<RenkuUrl>()
    }

    pub fn as_url(&self) -> &Url {
        let RenkuUrl(u) = self;
        u
    }

    pub fn as_str(&self) -> &str {
        let RenkuUrl(u) = self;
        u.as_str()
    }

    pub fn join(&self, seg: &str) -> Result<RenkuUrl, ParseError> {
        let RenkuUrl(u) = self;
        u.join(seg).map(RenkuUrl)
    }
}

impl<'de> Deserialize<'de> for RenkuUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let u = RenkuUrl::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(u)
    }
}

impl Serialize for RenkuUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let RenkuUrl(u) = self;
        serializer.serialize_str(u.as_str())
    }
}
impl Display for RenkuUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RenkuUrl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::parse(s).map(RenkuUrl)
    }
}
