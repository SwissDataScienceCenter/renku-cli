use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::cli::sink::Sink;
use crate::cli::BuildInfo;
use crate::httpclient::data::VersionInfo;
use crate::httpclient::Error as HttpError;
use clap::Parser;
use serde::Serialize;
use snafu::{ResultExt, Snafu};
use std::fmt;

/// Prints version about server and client.
///
/// Queries the server for its version information and prints more
/// version details about this client.
#[derive(Parser, Debug, PartialEq)]
pub struct Input {
    /// Only show the client version and don't request server side
    /// version information.
    #[arg(long, default_value_t = false)]
    pub client_only: bool,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Input {
    pub async fn exec<'a>(&self, ctx: &Context<'a>) -> Result<(), Error> {
        if self.client_only {
            let vinfo = BuildInfo::default();
            ctx.write_result(&vinfo).await.context(WriteResultSnafu)?;
        } else {
            let result = ctx
                .client
                .version(ctx.opts.verbose > 1)
                .await
                .context(HttpClientSnafu)?;
            let vinfo = Versions::create(result, &ctx.renku_url);
            ctx.write_result(&vinfo).await.context(WriteResultSnafu)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Versions<'a> {
    pub client: BuildInfo,
    pub server: VersionInfo,
    pub renku_url: &'a str,
}
impl Versions<'_> {
    pub fn create(server: VersionInfo, renku_url: &str) -> Versions {
        Versions {
            client: BuildInfo::default(),
            server,
            renku_url,
        }
    }
}

impl fmt::Display for Versions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hc = &self.server.search.head_commit[..8];
        write!(
            f,
            "Client:\n{}\n\nRenku @ {}\n  Data Services: {}\n  Search Services: {} ({})",
            self.client, self.renku_url, self.server.data.version, self.server.search.version, hc
        )
    }
}

impl Sink for Versions<'_> {}
impl Sink for BuildInfo {}
