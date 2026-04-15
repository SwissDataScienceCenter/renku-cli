pub mod list;
pub mod start;
pub mod stop;

use super::Context;
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error starting job: {}", source))]
    Start { source: start::Error },

    #[snafu(display("Error stopping job: {}", source))]
    Stop { source: stop::Error },

    #[snafu(display("Error listing jobs: {}", source))]
    List { source: list::Error },
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
            JobCommand::Stop(input) => input.exec(ctx).await.context(StopSnafu),
            JobCommand::List(input) => input.exec(ctx).await.context(ListSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum JobCommand {
    #[command()]
    Start(start::Input),

    #[command()]
    Stop(stop::Input),

    #[command()]
    List(list::Input),
}
