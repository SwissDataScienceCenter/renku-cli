use crate::data::simple_message::SimpleMessage;

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::httpclient::Error as HttpError;

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
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
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
        .context(WriteResultSnafu)?;
        Ok(())
    }
}
