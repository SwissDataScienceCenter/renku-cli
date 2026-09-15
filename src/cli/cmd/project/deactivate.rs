use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    httpclient::{self},
    project_config::{ProjectConfigError, RenkuProjectConfig},
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Unset currently active project [alias: d].
///
/// Removes global setting of which Renku project is currently active
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
    #[snafu(display("Config delete error: {}", source))]
    ConfigDelete { source: ProjectConfigError },
}

impl Input {
    pub async fn exec(&self, _ctx: Context) -> Result<(), Error> {
        RenkuProjectConfig::remove_global_config().context(ConfigDeleteSnafu)
    }
}
