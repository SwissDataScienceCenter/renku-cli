pub mod deposit;
pub mod zenodo;
mod zenodo_api;

use super::Context as ParentContext;
use clap::{Parser, ValueEnum};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Clone, ValueEnum)]
pub enum Provider {
    Zenodo,
}

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
        #[arg(long, default_value = "zenodo")]
        provider: Provider,
        #[command(subcommand)]
        cmd: DepositCommand,
    },
}

#[derive(Parser, Debug)]
pub enum DepositCommand {
    #[command(
        name = "copy",
        alias = "cp",
        about = "Copy and upload local files into the deposit"
    )]
    CopyFiles(deposit::CopyInput),
    #[command(name = "list", about = "List all deposits", alias = "ls")]
    ListDeposits(deposit::ListInput),
    #[command(name = "lsf", about = "List all files in a specific deposit")]
    ListFiles(deposit::ListFiles),
}

pub struct Context {
    pub parent: ParentContext,
    pub provider: Provider,
}

impl Context {
    pub fn new(ctx: ParentContext, provider: Provider) -> Context {
        Context {
            parent: ctx,
            provider,
        }
    }
}

impl Input {
    pub async fn exec(&self, ctx: ParentContext) -> Result<(), Error> {
        match &self.subcmd {
            DatasetCommand::Deposit { provider, cmd } => match cmd {
                DepositCommand::CopyFiles(input) => input
                    .exec(Context::new(ctx, provider.clone()))
                    .await
                    .context(DepositSnafu),
                DepositCommand::ListDeposits(input) => input
                    .exec(Context::new(ctx, provider.clone()))
                    .await
                    .context(DepositSnafu),
                DepositCommand::ListFiles(input) => input
                    .exec(Context::new(ctx, provider.clone()))
                    .await
                    .context(DepositSnafu),
            },
        }
    }
}
