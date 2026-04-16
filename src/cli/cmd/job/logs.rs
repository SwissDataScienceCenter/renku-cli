use super::Context;
use crate::httpclient::Error as HttpError;
use crate::{cli::sink::Error as SinkError, data::simple_message::SimpleMessage};

use clap::{Parser, ValueHint};

use snafu::{ResultExt, Snafu};

/// Listing logs of a jobs.
///
/// List the logs of a job.
#[derive(Parser, Debug)]
pub struct Input {
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
        let result = ctx
            .client
            .session_logs(&self.job_id)
            .await
            .context(HttpClientSnafu)?;

        if let Some(lines) = result.0.get("amalthea-session") {
            ctx.write_result(&SimpleMessage {
                message: lines.to_string(),
            })
            .await
            .context(WriteResultSnafu)?;
        } else {
            ctx.write_result(&SimpleMessage {
                message: "No logs available.".to_string(),
            })
            .await
            .context(WriteResultSnafu)?;
        }
        Ok(())
    }
}
