use crate::httpclient::data::ProjectDetails;

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::cli::sink::Sink;
use crate::httpclient::Error as HttpError;
use crate::util::data::ProjectId;

use clap::Parser;
use git2::Repository;
use snafu::{ResultExt, Snafu};
use std::fmt;
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
        let target = self.target_dir()?;
        clone_project(&details, &target)?;
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
fn clone_project(project: &ProjectDetails, parent: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(parent).context(CreateDirSnafu)?;
    for repo in project.repositories.iter() {
        let name = match repo.rsplit_once('/') {
            Some((_, n)) => n,
            None => "no-name",
        };
        let rr = Repository::clone(&repo, parent.join(name)).unwrap();
        println!("cloned: {:?}", rr.head().unwrap().name());
    }
    Ok(())
}

impl fmt::Display for ProjectDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .repositories
            .iter()
            .fold(String::new(), |a, b| a + "\n  - " + b);
        write!(
            f,
            "Id: {}\nNamespace/Slug: {}/{}\nVisibility: {}\nCreated At: {}\nRepositories:{}",
            self.id, self.namespace, self.slug, self.visibility, self.creation_date, lines
        )
    }
}
impl Sink for ProjectDetails {}
