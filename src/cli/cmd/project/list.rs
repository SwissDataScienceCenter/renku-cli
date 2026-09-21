use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    data::simple_message::SimpleMessage,
    httpclient::{self},
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing projects [alias: ls].
///
/// List all projects owned by a user
#[derive(Parser, Debug)]
pub struct Input {
    /// Get all projects, not just ones you're a member of
    #[arg(long, short, default_value_t = false)]
    pub all: bool,
    /// How many results to get
    #[arg(long, short, default_value_t = 100)]
    pub n_results: u16,
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
            .list_projects(!self.all, self.n_results)
            .await
            .context(HttpClientSnafu)?;
        if let Some(total_results) = result.1 {
            ctx.write_result(&SimpleMessage {
                message: format!("Showing {}/~{} results", result.0.0.len(), total_results),
            })
            .await
            .context(WriteResultSnafu)?;
        }

        ctx.write_result(&result.0).await.context(WriteResultSnafu)
    }
}
