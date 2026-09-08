use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    httpclient::{self, data::SessionMode},
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing launchers.
///
/// List currently running launchers.
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
        let mut result = ctx.client.list_launchers().await.context(HttpClientSnafu)?;

        result.retain(|e| e.launcher_type == SessionMode::NonInteractive);
        if let Ok(Some(project)) = ctx.resolve_project_context().await {
            result.retain(|v| v.project_id == project.id);
        }
        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
