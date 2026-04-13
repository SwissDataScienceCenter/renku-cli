use super::zenodo;
use super::{Context, Provider};
use clap::Parser;
use snafu::{ResultExt, Snafu};
use std::env::VarError;
use std::path::PathBuf;
use tabled::Table;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A dataset deposit error occured: {}", source))]
    Zenodo { source: zenodo::Error },

    #[snafu(display("Env variable error: {}", source))]
    EnvVarMissing { source: VarError },
}

/// Copies the data from a location into a data deposit
#[derive(Parser, Debug)]
pub struct CopyInput {
    /// The source directory where the files to be copied can be found.
    #[arg()]
    pub source_dir: PathBuf,

    /// The id of the deposit the data should be copied to.
    #[arg()]
    pub deposit_id: String,
}

impl CopyInput {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match ctx.provider {
            Provider::Zenodo => {
                let token = std::env::var("ZENODO_API_KEY").context(EnvVarMissingSnafu)?;
                let clnt = zenodo::ZenodoClient::new(
                    token,
                    ctx.parent
                        .opts
                        .verbosity
                        .log_level()
                        .unwrap_or(log::Level::Warn)
                        > log::Level::Info,
                );
                clnt.upload_files(&self.deposit_id, &self.source_dir)
                    .await
                    .context(ZenodoSnafu)
            }
        }
    }
}

/// List all depositions for the specific provider
#[derive(Parser, Debug)]
pub struct ListInput {}

impl ListInput {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match ctx.provider {
            Provider::Zenodo => {
                let token = std::env::var("ZENODO_API_KEY").context(EnvVarMissingSnafu)?;
                let clnt = zenodo::ZenodoClient::new(
                    token,
                    ctx.parent
                        .opts
                        .verbosity
                        .log_level()
                        .unwrap_or(log::Level::Warn)
                        > log::Level::Info,
                );
                let deps = clnt.get_depositions().await.context(ZenodoSnafu)?;
                println!("{}", Table::new(deps));
                Ok(())
            }
        }
    }
}

/// List all files in a specific deposit
#[derive(Parser, Debug)]
pub struct ListFiles {
    deposit_id: String,
}

impl ListFiles {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match ctx.provider {
            Provider::Zenodo => {
                let token = std::env::var("ZENODO_API_KEY").context(EnvVarMissingSnafu)?;
                let clnt = zenodo::ZenodoClient::new(
                    token,
                    ctx.parent
                        .opts
                        .verbosity
                        .log_level()
                        .unwrap_or(log::Level::Warn)
                        > log::Level::Info,
                );
                let files = clnt
                    .list_files(&self.deposit_id)
                    .await
                    .context(ZenodoSnafu)?;
                println!("{}", Table::new(files));
                Ok(())
            }
        }
    }
}
