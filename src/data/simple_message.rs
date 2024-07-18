use std::fmt;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SimpleMessage {
    pub message: String,
}

impl fmt::Display for SimpleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
