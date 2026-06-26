use crate::{
    cli::{cmd::job::logs, complete::complete_job_launcher_id},
    data::{simple_message::SimpleMessage, submission_id::SubmissionId},
    httpclient::{self, data::SessionStartRequest},
};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use ulid::Ulid;

use snafu::{ResultExt, Snafu};

/// Start a job.
///
/// Starts a non-interactive session using a pre-configured session launcher.
#[derive(Parser, Debug)]
pub struct Input {
    /// The launcher to use for launching the job.
    #[arg(long, value_hint=ValueHint::Other, add = ArgValueCompleter::new(complete_job_launcher_id))]
    pub launcher: Ulid,

    /// A submission id allows to deduplicate same job submissions. If
    /// missing, a random one is generated. It must be at least 4
    /// characters, starting with a lowercase letter, followed by
    /// alphanumeric characters (including the dash).
    #[arg(long)]
    pub submission_id: Option<SubmissionId>,

    /// Overwrite the command that is set in the launcher.
    #[arg(long)]
    pub command: Vec<String>,

    /// Start the job and show the logs until it ends or the user cancels with Ctrl-C.
    #[arg(long, default_value_t = false)]
    pub wait: bool,

    /// These arguments are passed to the renku job command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0.., value_name = "ARGS")]
    pub passthrough: Vec<String>,
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
        let submission_id = self
            .submission_id
            .clone()
            .unwrap_or_else(SubmissionId::random);
        let cmd = if self.command.is_empty() {
            None
        } else {
            Some(self.command.clone())
        };
        let args = if self.passthrough.is_empty() {
            None
        } else {
            Some(self.passthrough.clone())
        };
        let req = SessionStartRequest {
            launcher_id: self.launcher.to_string(),
            session_type: "non-interactive".into(),
            submission_id: Some(submission_id),
            job_args_override: args,
            job_command_override: cmd,
        };
        let result = ctx
            .client
            .start_session(req)
            .await
            .context(HttpClientSnafu)?;

        if self.wait {
            ctx.write_result(&SimpleMessage {
                message: format!(
                    "Started job {} (submission_id: {}). Waiting for logs...",
                    result.name,
                    result.submission_id.unwrap_or("-".to_string())
                ),
            })
            .await
            .context(WriteResultSnafu)?;
            let log_input = logs::Input {
                job_id: result.name,
                follow: true,
                follow_interval: 2,
            };
            log_input.follow_logs(ctx).await.context(HttpClientSnafu)
        } else {
            ctx.write_result(&result).await.context(WriteResultSnafu)
        }
    }
}
