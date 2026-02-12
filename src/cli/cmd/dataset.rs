pub mod deposit;
pub mod zenodo;

use super::Context;
use clap::Parser;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error with deposit: {}", source))]
    Deposit { source: deposit::Error },
}

/// Sub command for managing datasets
#[derive(Parser, Debug)]
pub struct Input {
    #[command(subcommand)]
    pub subcmd: DatasetCommand,
}

#[derive(Parser, Debug)]
pub enum DatasetCommand {
    Deposit {
        #[command(subcommand)]
        cmd: DepositCommand,
    },
}

#[derive(Parser, Debug)]
pub enum DepositCommand {
    #[command()]
    CopyFiles(deposit::CopyInput),
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match self.subcmd {
            DatasetCommand::Deposit { cmd: _ } => Ok(print!("Hi")),
        }
    }
}
