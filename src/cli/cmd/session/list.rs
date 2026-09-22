use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    httpclient::{
        self,
        data::{InteractiveSessionList, SessionMode},
    },
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Listing sessions [alias: ls].
///
/// List currently running sessions.
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
        let mut result = ctx
            .client
            .list_sessions(Some(SessionMode::Interactive))
            .await
            .context(HttpClientSnafu)?;

        if let Ok(Some(project)) = ctx.resolve_project_context().await {
            result.retain(|v| v.project_id == project.id);
        }
        let result = InteractiveSessionList(result);

        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
