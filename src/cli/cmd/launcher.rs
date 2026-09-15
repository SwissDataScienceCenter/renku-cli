pub mod list;

use super::Context;
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error listing launchers: {}", source))]
    List { source: list::Error },
}

/// Sub command for managing launchers [alias: l]
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: LauncherCommand,
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match &self.subcmd {
            LauncherCommand::List(input) => input.exec(ctx).await.context(ListSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum LauncherCommand {
    #[command(alias = "ls")]
    List(list::Input),
}
