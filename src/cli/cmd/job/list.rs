use super::Context;
use crate::httpclient::Error as HttpError;
use crate::{cli::sink::Error as SinkError, httpclient::data::SessionMode};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing jobs.
///
/// List currently running jobs.
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let result = ctx
            .client
            .list_sessions(Some(SessionMode::NonInteractive))
            .await
            .context(HttpClientSnafu)?;

        ctx.write_result(&result).await.context(WriteResultSnafu)?;
        Ok(())
    }
}
