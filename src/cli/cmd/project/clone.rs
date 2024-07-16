use crate::config::{ConfigError, ProjectInfo, RenkuProjectConfig};
use crate::httpclient::data::ProjectDetails;

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::httpclient::Error as HttpError;
use crate::util::data::{ProjectId, SimpleMessage};
use std::sync::Arc;

use clap::Parser;
use git2::{Error as GitError, Repository};
use snafu::{ResultExt, Snafu};
use std::path::{Path, PathBuf};
use tokio::task::{JoinError, JoinSet};

/// Clone a project.
///
/// Clones a renku project by creating a directory with the project
/// slug and cloning each code repository into it.
#[derive(Parser, Debug)]
pub struct Input {
    /// The project to clone, identified by either its id or the
    /// namespace/slug identifier.
    #[arg()]
    pub project_ref: String,

    /// Optional target directory to create the project in. By default
    /// the current working directory is used.
    #[arg()]
    pub target_dir: Option<String>,
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

    #[snafu(display("Error in task: {}", source))]
    TaskJoin { source: JoinError },

    #[snafu(display("Error creating config file: {}", source))]
    RenkuConfig { source: ConfigError },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let project_id = self.project_id()?;
        let opt_details = match &project_id {
            ProjectId::NamespaceSlug { namespace, slug } => ctx
                .client
                .get_project_by_slug(namespace, slug, ctx.opts.verbose > 1)
                .await
                .context(HttpClientSnafu)?,
            ProjectId::Id(id) => ctx
                .client
                .get_project_by_id(id, ctx.opts.verbose > 1)
                .await
                .context(HttpClientSnafu)?,
        };
        if let Some(details) = opt_details {
            let target = self.target_dir()?.join(&details.slug);
            let renku_project_cfg = RenkuProjectConfig {
                renku_url: ctx.renku_url.clone(),
                project: ProjectInfo {
                    id: details.id.clone(),
                    namespace: details.namespace.clone(),
                    slug: details.slug.clone(),
                },
            };
            ctx.write_err(&SimpleMessage {
                message: format!(
                    "Cloning {} ({}) into {}...",
                    details.slug,
                    details.id,
                    &target.display()
                ),
            })
            .await
            .context(WriteResultSnafu)?;

            write_config(renku_project_cfg, &target).await?;

            let ctx = clone_project(ctx, &details, target).await?;
            ctx.write_result(&details).await.context(WriteResultSnafu)?;
        } else {
            ctx.write_err(&SimpleMessage {
                message: format!("Project '{}' doesn't exist.", &project_id),
            })
            .await
            .context(WriteResultSnafu)?;
        }
        Ok(())
    }

    fn project_id(&self) -> Result<ProjectId, Error> {
        self.project_ref
            .parse::<ProjectId>()
            .context(ProjectIdParseSnafu)
    }

    fn target_dir(&self) -> Result<PathBuf, Error> {
        match &self.target_dir {
            Some(dir) => Ok(std::path::PathBuf::from(dir)),
            None => std::env::current_dir().context(CurrentDirSnafu),
        }
    }
}

async fn clone_project<'a>(
    ctx: Context,
    project: &ProjectDetails,
    target: PathBuf,
) -> Result<Context, Error> {
    tokio::fs::create_dir_all(&target)
        .await
        .context(CreateDirSnafu)?;

    let mut tasks = JoinSet::new();
    let cc = Arc::new(ctx);
    let tt = Arc::new(target);
    for repo in project.repositories.iter() {
        let cc = cc.clone();
        let tt = tt.clone();
        let rr = repo.to_string();
        tasks.spawn(clone_repository(cc, rr, tt));
    }

    while let Some(res) = tasks.join_next().await {
        res.context(TaskJoinSnafu)??;
    }
    Ok(Arc::into_inner(cc).unwrap())
}

async fn clone_repository(
    ctx: Arc<Context>,
    repo_url: String,
    dir: Arc<PathBuf>,
) -> Result<(), Error> {
    let name = match repo_url.rsplit_once('/') {
        Some((_, n)) => n,
        None => "no-name",
    };
    let local_path = dir.join(name);
    if local_path.exists() {
        ctx.write_err(&SimpleMessage {
            message: format!("The repository {} already exists", name),
        })
        .await
        .context(WriteResultSnafu)?;
    } else {
        // TODO use the repository builder to access more options,
        // show clone progress and provide credentials
        let (repo, repo_url, local_path) = tokio::task::spawn_blocking(|| {
            let r = Repository::clone(&repo_url, &local_path).context(GitCloneSnafu);
            (r, repo_url, local_path)
        })
        .await
        .context(TaskJoinSnafu)?;
        let git_repo = repo?;
        if ctx.opts.verbose > 1 {
            let head = git_repo
                .head()
                .ok()
                .and_then(|r| r.name().map(str::to_string));
            log::debug!("Checked out ref {:?} for repo {}", head, repo_url);
        }

        ctx.write_err(&SimpleMessage {
            message: format!("Cloned: {} to {}", repo_url, local_path.display()),
        })
        .await
        .context(WriteResultSnafu)?;
    }
    Ok(())
}

async fn write_config(data: RenkuProjectConfig, local_dir: &Path) -> Result<(), Error> {
    let target = local_dir.join(".renku").join("config.toml");
    tokio::task::spawn_blocking(move || data.write(&target).context(RenkuConfigSnafu))
        .await
        .context(TaskJoinSnafu)?
}
