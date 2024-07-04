use crate::httpclient::data::ProjectDetails;

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::httpclient::Error as HttpError;
use crate::util::data::{ProjectId, SimpleMessage};

use clap::Parser;
use git2::{Error as GitError, Repository};
use snafu::{ResultExt, Snafu};
use std::path::Path;
use std::path::PathBuf;

/// Clone a project
#[derive(Parser, Debug)]
pub struct Input {
    /// The first argument is the project to clone, identified by
    /// either its id or the namespace/slug identifier. The second
    /// argument is optional, defining the target directory to create
    /// the project in.
    #[arg(required = true, num_args = 1..=2)]
    pub project_and_target: Vec<String>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Error reading project id: {}", source))]
    ProjectIdParse {
        source: crate::util::data::ProjectIdParseError,
    },

    #[snafu(display("Error getting current directory: {}", source))]
    CurrentDir { source: std::io::Error },

    #[snafu(display("Error creating directory: {}", source))]
    CreateDir { source: std::io::Error },

    #[snafu(display("Error cloning project: {}", source))]
    GitClone { source: GitError },
}

impl Input {
    pub async fn exec<'a>(&self, ctx: &Context<'a>) -> Result<(), Error> {
        let details = match self.project_id()? {
            ProjectId::NamespaceSlug { namespace, slug } => ctx
                .client
                .get_project_by_slug(&namespace, &slug, ctx.opts.verbose > 1)
                .await
                .context(HttpClientSnafu)?,
            ProjectId::Id(id) => ctx
                .client
                .get_project_by_id(&id, ctx.opts.verbose > 1)
                .await
                .context(HttpClientSnafu)?,
        };
        let target = self.target_dir()?.join(&details.slug);
        ctx.write_err(&SimpleMessage {
            message: format!(
                "Cloning {} ({}) into {}...",
                details.slug,
                details.id,
                target.display()
            ),
        })
        .await
        .context(WriteResultSnafu)?;

        clone_project(ctx, &details, &target).await?;
        ctx.write_result(&details).await.context(WriteResultSnafu)?;
        Ok(())
    }

    fn project_id(&self) -> Result<ProjectId, Error> {
        self.project_and_target
            .first()
            .unwrap() // clap makes sure there is at least one element (🤞)
            .parse::<ProjectId>()
            .context(ProjectIdParseSnafu)
    }

    fn target_dir(&self) -> Result<PathBuf, Error> {
        match self.project_and_target.get(1) {
            Some(dir) => Ok(std::path::PathBuf::from(dir)),
            None => std::env::current_dir().context(CurrentDirSnafu),
        }
    }
}

//TODO make async
async fn clone_project<'a>(
    ctx: &Context<'a>,
    project: &ProjectDetails,
    target: &Path,
) -> Result<(), Error> {
    std::fs::create_dir_all(target).context(CreateDirSnafu)?;
    for repo in project.repositories.iter() {
        clone_repository(ctx, &repo, target).await?;
    }
    Ok(())
}

async fn clone_repository<'a>(ctx: &Context<'a>, repo_url: &str, dir: &Path) -> Result<(), Error> {
    let name = match repo_url.rsplit_once('/') {
        Some((_, n)) => n,
        None => "no-name",
    };
    let local_path = dir.join(&name);
    if local_path.exists() {
        ctx.write_err(&SimpleMessage {
            message: format!("The repository {} already exists", name),
        })
        .await
        .context(WriteResultSnafu)?;
    } else {
        //TODO use the builder to access more options
        Repository::clone(&repo_url, &local_path).context(GitCloneSnafu)?;
        ctx.write_err(&SimpleMessage {
            message: format!("Cloned: {} to {}", repo_url, local_path.display()),
        })
        .await
        .context(WriteResultSnafu)?;
    }
    Ok(())
}
