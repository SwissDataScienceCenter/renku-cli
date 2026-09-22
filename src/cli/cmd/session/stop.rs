use crate::{
    cli::complete::complete_interactive_session_name, data::simple_message::SimpleMessage,
    httpclient,
};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};

use clap_complete::ArgValueCompleter;
use snafu::{ResultExt, Snafu};

/// Stop a session.
///
/// Stop a running interactive session.
#[derive(Parser, Debug)]
pub struct Input {
    /// The id of the session to stop
    #[arg(value_hint=ValueHint::Other, add = ArgValueCompleter::new(complete_interactive_session_name))]
    pub session_id: String,
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
            .stop_session(&self.session_id)
            .await
            .context(HttpClientSnafu)?;
        ctx.write_result(&SimpleMessage {
            message: "Session is being removed.".into(),
        })
        .await
        .context(WriteResultSnafu)
    }
}
