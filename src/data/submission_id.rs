use std::{fmt::Display, str::FromStr};

use regex_macro::regex;
use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Debug, PartialEq, Clone)]
pub struct SubmissionId(String);

impl SubmissionId {
    pub fn parse<S: AsRef<str>>(input: S) -> Result<SubmissionId, SubmissionIdError> {
        let s = input.as_ref();
        if regex!("^[a-z][-0-9a-z]{3,19}$").is_match(s) {
            Ok(SubmissionId(s.into()))
        } else {
            Err(SubmissionIdError::InvalidInput(s.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        let SubmissionId(u) = self;
        u.as_str()
    }

    pub fn join(&self, seg: &str) -> Result<SubmissionId, SubmissionIdError> {
        let SubmissionId(u) = self;
        SubmissionId::parse(format!("{}{}", u, seg))
    }

    pub fn random() -> SubmissionId {
        let first = util::strings::random(1, "abcdefghijklmnopqrstuvwxyz");
        let s = util::strings::random_lower_alpha_num(8);
        SubmissionId(format!("{}{}", first, s))
    }
}

impl<'de> Deserialize<'de> for SubmissionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let u = SubmissionId::parse(&s).map_err(serde::de::Error::custom)?;
        Ok(u)
    }
}

impl Serialize for SubmissionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let SubmissionId(u) = self;
        serializer.serialize_str(u.as_str())
    }
}
impl Display for SubmissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SubmissionId {
    type Err = SubmissionIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SubmissionId::parse(s)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SubmissionIdError {
    InvalidInput(String),
}

impl Display for SubmissionIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionIdError::InvalidInput(msg) => write!(f, "Invalid submission id: {}", msg),
        }
    }
}
impl std::error::Error for SubmissionIdError {}

#[test]
fn submission_id_parse() {
    assert!(SubmissionId::parse("__-").is_err());
    assert!(SubmissionId::parse("9abcd").is_err());
    assert!(SubmissionId::parse("aBCDEFg").is_err());
    assert!(SubmissionId::parse("abc-9ed").is_ok());
    assert_eq!(
        SubmissionId::parse("ab-cd-de").unwrap().as_str(),
        "ab-cd-de"
    );
    assert!(SubmissionId::random().as_str().len() > 4);
}
