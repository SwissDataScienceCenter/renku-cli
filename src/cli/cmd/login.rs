use std::path::{Path, PathBuf};

use super::Context;
use crate::httpclient::auth::{Response, UserCode};
use crate::httpclient::Error as HttpError;
use crate::{cli::sink::Error as SinkError, data::simple_message::SimpleMessage};
use clap::{Parser, ValueHint};

use snafu::{ResultExt, Snafu};

/// Performs a login to renku.
///
/// The login consists of two parts:
///
/// 1. Renku is queried to return a temporary URL that can be used to
/// authenticate and authorize this application. The url must be
/// opened with some device and the user code must be entered (if
/// necessary).
///
/// 2. Once the first step is complete, the cli can obtain an access
/// token and does so by periodically polling the renku platform.
///
/// The login command can do these two steps separately. This requires
/// to run with `--user-code-only` to omit the second step and store
/// the JSON output to some file. Later, run with `--continue-from`
/// specifying the path to that file to continue the login process.
///
/// When the token is received, it is stored in the application data
/// folder of your system. Once it expires, the login process must be
/// repeated.
///
/// The access token can also be manually given as an environment
/// variable RENKU_CLI_ACCESS_TOKEN (then the login command is not
/// required and its result will be ignored).
#[derive(Parser, Debug, PartialEq)]
pub struct Input {
    /// Do not poll for the access token, only print the user code information.
    #[clap(long, default_value_t = false, group = "steps")]
    pub user_code_only: bool,

    /// Given the (json) output of the first step, continue by polling
    /// for the access token.
    #[clap(long, value_hint = ValueHint::FilePath, group = "steps")]
    pub continue_from: Option<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Reading file {} failed: {}", file.display(), source))]
    FileRead {
        source: std::io::Error,
        file: PathBuf,
    },
    #[snafu(display("Error decoding user code data: {}", source))]
    JsonDecode { source: serde_json::Error },
}

#[derive(Clone, Debug, PartialEq)]
enum Steps<'a> {
    UserCode,
    Continue(&'a Path),
    Complete,
}
impl Input {
    fn get_steps(&self) -> Steps {
        if let Some(p) = &self.continue_from {
            Steps::Continue(p)
        } else if self.user_code_only {
            Steps::UserCode
        } else {
            Steps::Complete
        }
    }
}

impl Input {
    pub async fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let steps = self.get_steps();
        if let Steps::Continue(file) = &steps {
            let buf = tokio::fs::read(file)
                .await
                .context(FileReadSnafu { file })?;
            let info = serde_json::from_slice::<UserCode>(&buf).context(JsonDecodeSnafu)?;
            let resp = ctx
                .client
                .complete_login_flow(info)
                .await
                .context(HttpClientSnafu)?;

            print_success(ctx, &resp).await?;
        } else {
            let info = ctx
                .client
                .start_login_flow()
                .await
                .context(HttpClientSnafu)?;

            ctx.write_result(&info).await.context(WriteResultSnafu)?;

            if steps == Steps::Complete {
                ctx.write_result(&SimpleMessage {
                    message: "Waiting for authorization response…".into(),
                })
                .await
                .context(WriteResultSnafu)?;
                let resp = ctx
                    .client
                    .complete_login_flow(info)
                    .await
                    .context(HttpClientSnafu)?;

                print_success(ctx, &resp).await?;
            }
        }
        Ok(())
    }
}

async fn print_success(ctx: &Context, resp: &Response) -> Result<(), Error> {
    ctx.write_result(resp).await.context(WriteResultSnafu)
}
