pub mod clone;

use super::{Cmd, Context};
use clap::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    Clone { source: clone::Error },
}

/// Sub command for managing projects
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: ProjectCommand,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        match &self.subcmd {
            ProjectCommand::Clone(input) => input.exec(ctx).context(CloneSnafu),
        }
    }
}

#[derive(Parser, Debug)]
pub enum ProjectCommand {
    #[command()]
    Clone(clone::Input),
}
