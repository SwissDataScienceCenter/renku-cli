use std::fmt;

use crate::cli::sink::{Error as SinkError, Sink};

use super::Context;
use clap::Parser;
use self_update::{cargo_crate_version, errors::Error as SelfUpdateError};
use serde::Serialize;
use snafu::{ResultExt, Snafu};
use tokio::task::JoinError;

/// Checks if a new version is available and performs self update.
///
/// Queries Github for available releases
#[derive(Parser, Debug, PartialEq)]
pub struct Input {}

fn update() -> Result<UpdateResult, Error> {
    let result = self_update::backends::github::Update::configure()
        .repo_owner("SwissDataScienceCenter")
        .repo_name("renku-cli")
        .bin_name("rnk")
        .bin_path_in_archive("rnk")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .context(BuildSnafu)?
        .update();
    match result {
        Ok(status) => {
            let version = status.version().to_owned();
            if status.updated() {
                Ok(UpdateResult::Updated(version))
            } else {
                Ok(UpdateResult::AlreadyUpToDate(version))
            }
        }
        Err(SelfUpdateError::Update(msg)) if msg == "Update aborted" => Ok(UpdateResult::Aborted),
        Err(e) => Err(Error::Update { source: e }),
    }
}

impl Input {
    pub async fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let status = tokio::task::spawn_blocking(update)
            .await
            .context(JoinSnafu)??;
        ctx.write_result(&status).await.context(WriteResultSnafu)
    }
}
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Couldn't perform update: {}", source))]
    Update { source: SelfUpdateError },

    #[snafu(display("Couldn't build self updater: {}", source))]
    Build { source: SelfUpdateError },

    #[snafu(display("Couldn't join future: {}", source))]
    Join { source: JoinError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}
#[derive(Debug, Serialize)]
pub enum UpdateResult {
    Updated(String),
    AlreadyUpToDate(String),
    Aborted,
}

impl fmt::Display for UpdateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Updated(version) => {
                write!(f, "Successfully updated to version: {}", version)
            }
            Self::AlreadyUpToDate(version) => {
                write!(f, "Already up to date at version: {}", version)
            }
            Self::Aborted => Ok(()),
        }
    }
}

impl Sink for UpdateResult {}
