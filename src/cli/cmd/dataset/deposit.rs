use super::zenodo;
use super::{Context, Provider};
use clap::Parser;
use snafu::{ResultExt, Snafu};
use std::env::VarError;
use std::path::PathBuf;
use tabled::builder::Builder;

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
                let clnt = zenodo::ZenodoClient::new(token, ctx.parent.opts.verbose > 1);
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
                let clnt = zenodo::ZenodoClient::new(token, ctx.parent.opts.verbose > 1);
                let deps = clnt.get_depositions().await.context(ZenodoSnafu)?;
                let mut table = Builder::default();
                table.push_record(["ID", "Title", "State", "Created at"]);
                for d in deps {
                    table.push_record([
                        d.id.to_string(),
                        d.title,
                        d.state,
                        d.created
                            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
                    ]);
                }
                println!("{}", table.build());
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
                let clnt = zenodo::ZenodoClient::new(token, ctx.parent.opts.verbose > 1);
                let files = clnt
                    .list_files(&self.deposit_id)
                    .await
                    .context(ZenodoSnafu)?;
                let mut table = Builder::default();
                table.push_record(["Filename", "Size", "Checksum"]);
                for f in files {
                    table.push_record([f.filename, f.filesize.to_string(), f.checksum]);
                }
                println!("{}", table.build());
                Ok(())
            }
        }
    }
}
