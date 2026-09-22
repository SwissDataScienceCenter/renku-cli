pub mod list;
pub mod logs;
pub mod start;
pub mod stop;

use super::Context;
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error starting session: {}", source))]
    Start { source: start::Error },

    #[snafu(display("Error stopping session: {}", source))]
    Stop { source: stop::Error },

    #[snafu(display("Error listing sessions: {}", source))]
    List { source: list::Error },

    #[snafu(display("Error getting logs: {}", source))]
    Logs { source: logs::Error },
}

/// Sub command for managing sessions [alias: s]
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: SessionCommand,
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match &self.subcmd {
            SessionCommand::Start(input) => input.exec(ctx).await.context(StartSnafu),
            SessionCommand::Stop(input) => input.exec(ctx).await.context(StopSnafu),
            SessionCommand::List(input) => input.exec(ctx).await.context(ListSnafu),
            SessionCommand::Logs(input) => input.exec(ctx).await.context(LogsSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum SessionCommand {
    #[command()]
    Start(start::Input),

    #[command()]
    Stop(stop::Input),

    #[command(alias = "ls")]
    List(list::Input),

    #[command()]
    Logs(logs::Input),
}
