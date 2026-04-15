use crate::{data::simple_message::SimpleMessage, project_config::ProjectConfigError};

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::data::project_id::ProjectIdParseError;
use crate::httpclient::Error as HttpError;

use clap::{Parser, ValueHint};
use git2::Error as GitError;
use tokio::task::JoinError;
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
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Error reading project id: {}", source))]
    ProjectIdParse { source: ProjectIdParseError },

    #[snafu(display("Error getting current directory: {}", source))]
    CurrentDir { source: std::io::Error },

    #[snafu(display("Error creating directory: {}", source))]
    CreateDir { source: std::io::Error },

    #[snafu(display("Error cloning project: {}", source))]
    GitClone { source: GitError },

    #[snafu(display("Error in task: {}", source))]
    TaskJoin { source: JoinError },

    #[snafu(display("Error creating config file: {}", source))]
    RenkuConfig { source: ProjectConfigError },

    #[snafu(display("The project name is missing: {}", repo_url))]
    MissingProjectName { repo_url: String },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        ctx.write_result(&SimpleMessage {
            message: "Hello world".into(),
        })
        .await
        .context(WriteResultSnafu)?;
        Ok(())
    }
}
