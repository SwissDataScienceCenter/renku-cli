use super::Context;
use crate::httpclient::Error as HttpError;
use crate::{
    cli::sink::Error as SinkError, data::simple_message::SimpleMessage,
};
use clap::Parser;
use snafu::{ResultExt, Snafu};

/// Performs a logout by removing the stored token from the keystore.
///
#[derive(Parser, Debug, PartialEq)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Input {
    pub async fn exec(&self, ctx: &Context) -> Result<(), Error> {
        ctx.client.clear_token().await.context(HttpClientSnafu)?;
        let message = "Logout complete.".to_string();
        ctx.write_result(&SimpleMessage { message })
            .await
            .context(WriteResultSnafu)?;
        Ok(())
    }
}
