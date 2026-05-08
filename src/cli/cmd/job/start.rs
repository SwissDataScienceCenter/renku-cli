use crate::httpclient::{self, data::SessionStartRequest};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};
use ulid::Ulid;

use snafu::{ResultExt, Snafu};

/// Start a job.
///
/// Starts a non-interactive session using a pre-configured session launcher.
#[derive(Parser, Debug)]
pub struct Input {
    /// The launcher to use for launching the job.
    #[arg(value_hint=ValueHint::Other)]
    pub launcher: Ulid,
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
        let req = SessionStartRequest {
            launcher_id: self.launcher.to_string(),
            session_type: "non-interactive".into(),
        };
        let result = ctx
            .client
            .start_session(req)
            .await
            .context(HttpClientSnafu)?;

        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
