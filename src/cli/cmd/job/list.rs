use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    httpclient::{self, data::SessionMode},
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing jobs.
///
/// List currently running jobs.
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let result = ctx
            .client
            .list_sessions(Some(SessionMode::NonInteractive))
            .await
            .context(HttpClientSnafu)?;

        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
