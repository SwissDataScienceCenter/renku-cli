//! Global error types

use crate::cli::cmd;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    Cmd { source: cmd::CmdError },
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<cmd::CmdError> for Error {
    fn from(e: cmd::CmdError) -> Error {
        Error::Cmd { source: e }
    }
}
