use super::{Cmd, Context};
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
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = ctx
            .client
            .version(ctx.opts.verbose > 1)
            .context(HttpClientSnafu)?;
        let vinfo = Versions::create(result, &ctx.renku_url);
        ctx.write_result(vinfo).context(WriteResultSnafu)?;
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
