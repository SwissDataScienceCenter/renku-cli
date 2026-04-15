pub mod start;

use super::Context;
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error starting job: {}", source))]
    Start { source: start::Error },
}

/// Sub command for managing projects
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: JobCommand,
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match &self.subcmd {
            JobCommand::Start(input) => input.exec(ctx).await.context(StartSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum JobCommand {
    #[command()]
    Start(start::Input),
}
