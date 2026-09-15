use super::Context;
use crate::{
    cli::{opts::ProjectContextError, sink::Error as SinkError},
    httpclient::{self},
    project_config::ProjectConfigError,
};

use clap::Parser;

use snafu::{ResultExt, Snafu};

/// Shows the currently active project [alias: c].
///
/// Shows which project is currently active, taking into account the precedence
/// order of --project_context flag, RENKU_CLI_PROJECT_CONTEXT env var, local
/// project config file and globally active project (through the
/// `rnk project activate` command).
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
    #[snafu(display("Config delete error: {}", source))]
    ConfigDelete { source: ProjectConfigError },
    #[snafu(display("Project context error: {}", source))]
    ProjectContext { source: ProjectContextError },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let client = ctx.opts.create_client(None).context(HttpClientSnafu)?;
        let Some(project_id) = ctx
            .opts
            .get_project_context()
            .context(ProjectContextSnafu)?
        else {
            return Ok(());
        };
        let Some(project) = client
            .get_project(&project_id)
            .await
            .context(HttpClientSnafu)?
        else {
            return Ok(());
        };
        ctx.write_result(&project).await.context(WriteResultSnafu)
    }
}
