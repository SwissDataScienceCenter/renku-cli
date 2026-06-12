use super::Context;
use crate::cli::BuildInfo;
use crate::cli::sink::Error as SinkError;
use crate::cli::sink::Sink;
use crate::httpclient::Error as HttpError;
use crate::httpclient::data::VersionInfo;
use clap::Parser;
use serde::Serialize;
use snafu::{ResultExt, Snafu};
use std::fmt;

/// Prints version about server and client.
///
/// Prints version details about this client and can also query the renku platform for its verion.
#[derive(Parser, Debug, PartialEq)]
pub struct Input {
    /// Also request the version on the renku platform.
    #[arg(long, default_value_t = false)]
    pub with_server: bool,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Input {
    pub async fn exec(&self, ctx: &Context) -> Result<(), Error> {
        if self.with_server {
            let result = ctx.client.version().await.context(HttpClientSnafu)?;
            let info = Versions::create(result);
            ctx.write_result(&info).await.context(WriteResultSnafu)?;
        } else {
            let build_info = BuildInfo::default();
            ctx.write_result(&build_info)
                .await
                .context(WriteResultSnafu)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Versions {
    pub renku_cli: BuildInfo,
    pub renku_platform: VersionInfo,
}
impl Versions {
    pub fn create(renku_platform: VersionInfo) -> Versions {
        Versions {
            renku_cli: BuildInfo::default(),
            renku_platform,
        }
    }
}

impl fmt::Display for Versions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n\n{}", self.renku_cli, self.renku_platform)
    }
}

impl Sink for Versions {}
