use super::Context;

use clap::Parser;
use snafu::Snafu;

/// Clone a project
#[derive(Parser, Debug)]
pub struct Input {
    /// The project slug
    pub slug: String,
}

impl Input {
    pub async fn exec<'a>(&self, _ctx: &Context<'a>) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    Dummy,
}
