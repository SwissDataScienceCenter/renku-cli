use super::{Cmd, Context};

use clap::Parser;
use snafu::Snafu;

/// Clone a project
#[derive(Parser, Debug)]
pub struct Input {
    /// The project slug
    pub slug: String,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    Dummy,
}
