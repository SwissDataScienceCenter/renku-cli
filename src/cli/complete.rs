use std::ffi;

use crate::{
    cli::opts::MainOpts,
    data::project_id::ProjectId,
    httpclient::{
        Client,
        data::{SessionLauncher, SessionMode},
    },
};

use clap::{Parser, builder::StyledStr, error::Error as ClapError};
use clap_complete::CompletionCandidate;

use super::opts::CommonOpts;

// Helper function to create completion-candidate functions that are
// async and use the client and common options for their
// implementation.
fn make_sync_completer<F, Fut>(current: &ffi::OsStr, func: F) -> Vec<CompletionCandidate>
where
    F: Fn(Client, CommonOpts) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Vec<CompletionCandidate>> + Send + 'static,
{
    let Some(_current) = current.to_str() else {
        return vec![];
    };

    let Ok(opts) = parse_common_opts() else {
        eprintln!("Completions failed: Error parsing common options");
        return vec![];
    };

    let Ok(client) = opts.create_client(None) else {
        eprintln!("Completions failed: Error creating http renku client");
        return vec![];
    };

    tokio::task::block_in_place(move || {
        tokio::runtime::Handle::current().block_on(func(client, opts))
    })
}

/// Parses the part of the arguments that make up CommonOpts.
fn parse_common_opts() -> Result<CommonOpts, ClapError> {
    // this is a bit nasty, due to lack of a better option: manually
    // massage the arguments to remove everything after the first
    // non-option argument appears, which is the subcommand passed to
    // the binary. Then the standard command 'version' is appended, so
    // that parsing succeeds. Only common-options are of interest
    // here.
    let mut it = std::env::args().skip(2);
    let first = it.next();
    let first_it = first.map(std::iter::once).into_iter().flatten();
    let remain = it.take_while(|e| e.starts_with('-'));
    // the version command to make arg parsing successful
    let version = std::iter::once("version".to_string());
    let args = first_it.chain(remain).chain(version);
    MainOpts::try_parse_from(args).map(|e| e.common_opts)
}

async fn make_launcher_completion_candidate(
    client: &Client,
    launcher: &SessionLauncher,
) -> CompletionCandidate {
    let mut help = StyledStr::new();
    help.push_str(&launcher.name);
    let cc = CompletionCandidate::new(launcher.id.clone());

    let Ok(Some(project)) = client.get_project_by_id(&launcher.project_id).await else {
        eprintln!("Cannot get project details for: {}", launcher.project_id);
        return cc.help(Some(help));
    };

    help.push_str(" - ");
    help.push_str(&project.name);

    cc.help(Some(help))
}

async fn resolve_project_id(client: &Client, id: ProjectId) -> Option<String> {
    match client.get_project(&id).await {
        Ok(Some(p)) => Some(p.id),
        Ok(None) => {
            eprintln!("Project context not found: {}", id);
            None
        }
        Err(msg) => {
            eprintln!("Error getting project for id '{}': {}", id, msg);
            None
        }
    }
}

/// Complete a job session launcher id
pub fn complete_job_launcher_id(current: &ffi::OsStr) -> Vec<CompletionCandidate> {
    make_sync_completer(current, async |client, opts| {
        let Ok(launchers) = client.list_launchers().await else {
            eprintln!("Completions failed: Error getting list of launchers");
            return vec![];
        };
        let mut result: Vec<CompletionCandidate> = vec![];
        let project_ctx = opts.get_project_context().ok().flatten();
        let project_id = match project_ctx {
            Some(id) => resolve_project_id(&client, id).await,
            None => None,
        };
        for launcher in launchers
            .iter()
            .filter(|e| e.launcher_type == SessionMode::NonInteractive)
            .filter(|e| match &project_id {
                Some(id) => id == &e.project_id,
                None => true,
            })
        {
            let cc = make_launcher_completion_candidate(&client, launcher).await;
            result.push(cc);
        }
        if result.is_empty() {
            eprintln!("No job launchers found.");
        }
        result
    })
}
