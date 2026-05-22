use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Debug, PartialEq, Clone)]
pub struct SubmissionId(String);

impl SubmissionId {
    pub fn parse<S: AsRef<str>>(input: S) -> Result<SubmissionId, SubmissionIdError> {
        let s = input.as_ref();
        if s.len() < 4 {
            Err(SubmissionIdError::InvalidInput(s.to_string()))
        } else {
            Ok(SubmissionId(s.into()))
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
        let s = util::strings::random_alpha_num(8);
        SubmissionId(s)
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
