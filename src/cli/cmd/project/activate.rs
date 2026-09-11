use super::Context;
use crate::{
    cli::sink::Error as SinkError,
    data::{project_id::ProjectId, simple_message::SimpleMessage},
    httpclient::{self},
    project_config::{ProjectConfigError, ProjectInfo, RenkuProjectConfig},
};

use clap::{Parser, ValueHint};

use snafu::{ResultExt, Snafu};

/// Set a project as the currently active one [alias: a].
///
/// Limits other commands to work only on the currently active project.
#[derive(Parser, Debug)]
pub struct Input {
    /// The project to set as active, identified by either its id, the
    /// namespace/slug identifier or the complete url.
    #[arg(value_hint=ValueHint::Other)]
    pub project_ref: ProjectId,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
    #[snafu(display("Config write error: {}", source))]
    ConfigWrite { source: ProjectConfigError },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let opt_details = ctx
            .client
            .get_project(&self.project_ref)
            .await
            .context(HttpClientSnafu)?;
        if let Some(details) = opt_details {
            let cfg = RenkuProjectConfig::new(
                ctx.renku_url().clone(),
                ProjectInfo {
                    id: details.id.clone(),
                    namespace: details.namespace.clone(),
                    slug: details.slug.clone(),
                },
            );
            cfg.write_global_config().context(ConfigWriteSnafu)?;
        } else {
            ctx.write_err(&SimpleMessage {
                message: format!("Project '{}' doesn't exist.", self.project_ref),
            })
            .await
            .context(WriteResultSnafu)?;
        }
        Ok(())
    }
}
