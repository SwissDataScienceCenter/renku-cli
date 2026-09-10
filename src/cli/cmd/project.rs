pub mod clone;
pub mod list;

use super::Context;
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error cloning project: {}", source))]
    Clone { source: clone::Error },
    #[snafu(display("Error listing projects: {}", source))]
    List { source: list::Error },
}

/// Sub command for managing projects [alias: p]
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: ProjectCommand,
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match &self.subcmd {
            ProjectCommand::Clone(input) => input.exec(ctx).await.context(CloneSnafu),
            ProjectCommand::List(input) => input.exec(ctx).await.context(ListSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum ProjectCommand {
    #[command()]
    Clone(clone::Input),
    #[command(alias = "ls")]
    List(list::Input),
}
