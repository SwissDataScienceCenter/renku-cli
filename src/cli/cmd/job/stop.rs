use crate::{data::simple_message::SimpleMessage, httpclient};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};

use snafu::{ResultExt, Snafu};

/// Stop a job.
///
/// Stop a running non-interactive session.
#[derive(Parser, Debug)]
pub struct Input {
    /// The launcher to use for launching the job.
    #[arg(value_hint=ValueHint::Other)]
    pub job_id: String,
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
        ctx.client
            .stop_session(&self.job_id)
            .await
            .context(HttpClientSnafu)?;
        ctx.write_result(&SimpleMessage {
            message: "Job is being removed.".into(),
        })
        .await
        .context(WriteResultSnafu)
    }
}
