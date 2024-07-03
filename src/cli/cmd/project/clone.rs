use crate::httpclient::data::ProjectDetails;

use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::cli::sink::Sink;
use crate::httpclient::Error as HttpError;
use crate::util::data::ProjectId;

use clap::Parser;
use snafu::{ResultExt, Snafu};
use std::fmt;
//use std::path::Path;

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
}

// fn prepare_directory(project: &ProjectDetails, parent: &Path) -> Result<(), Error> {
//     Ok(())
// }

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
