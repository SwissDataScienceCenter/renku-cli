use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    httpclient::{self},
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing projects.
///
/// List all projects owned by a user
#[derive(Parser, Debug)]
pub struct Input {
    /// Get all projects, not just ones you're a member of
    #[arg(long, short, default_value_t = false)]
    pub all: bool,
}

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
            .list_projects(!self.all)
            .await
            .context(HttpClientSnafu)?;

        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
