use crate::{
    cli::{cmd::session::logs, complete::complete_session_launcher_id},
    data::simple_message::SimpleMessage,
    httpclient::{self, data::SessionStartRequest},
};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use ulid::Ulid;

use snafu::{ResultExt, Snafu};

/// Start a session.
///
/// Starts an interactive session using a pre-configured session launcher.
#[derive(Parser, Debug)]
pub struct Input {
    /// The launcher to use for launching the session.
    #[arg(long, value_hint=ValueHint::Other, add = ArgValueCompleter::new(complete_session_launcher_id))]
    pub launcher: Ulid,

    /// Start the session and show the logs until it ends or the user cancels with Ctrl-C.
    #[arg(long, default_value_t = false)]
    pub wait: bool,
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
            session_type: "interactive".into(),
            ..Default::default()
        };
        let result = ctx
            .client
            .start_session(req)
            .await
            .context(HttpClientSnafu)?;

        if self.wait {
            ctx.write_result(&SimpleMessage {
                message: format!("Started session {}. Waiting for logs...", result.name,),
            })
            .await
            .context(WriteResultSnafu)?;
            let log_input = logs::Input {
                session_id: result.name,
                follow: true,
                follow_interval: 2,
            };
            log_input.follow_logs(ctx).await.context(HttpClientSnafu)
        } else {
            ctx.write_result(&result).await.context(WriteResultSnafu)
        }
    }
}
