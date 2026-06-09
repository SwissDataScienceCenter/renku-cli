use std::{ffi, thread};

use crate::{
    cli::opts::MainOpts,
    httpclient::{
        Client,
        data::{SessionLauncher, SessionMode},
    },
};

use clap::{Parser, builder::StyledStr};
use clap_complete::CompletionCandidate;
use futures::executor::block_on;
use tokio::sync::mpsc;

use super::opts::CommonOpts;

pub fn make_sync_completer<F, Fut>(func: F) -> impl Fn(&ffi::OsStr) -> Vec<CompletionCandidate>
where
    F: Fn(&ffi::OsStr, Client) -> Fut + Clone + 'static,
    Fut: Future<Output = Vec<CompletionCandidate>> + 'static,
{
    move |current: &ffi::OsStr| {
        let Some(_current) = current.to_str() else {
            return vec![];
        };

        let args = truncate_to_common_opts(std::env::args());
        let Ok(opts) = MainOpts::try_parse_from(args) else {
            return vec![];
        };

        let Ok(client) = opts.common_opts.create_client(None) else {
            return vec![];
        };

        block_on(func(current, client))
    }
}

pub fn make_sync_completer2<F, Fut>(current: &ffi::OsStr, func: F) -> Vec<CompletionCandidate>
where
    F: Fn(Client, CommonOpts) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Vec<CompletionCandidate>> + Send + 'static,
{
    let Some(_current) = current.to_str() else {
        return vec![];
    };

    let args = truncate_to_common_opts(std::env::args()).collect::<Vec<String>>();
    let Ok(opts) = MainOpts::try_parse_from(args) else {
        return vec![];
    };

    let Ok(client) = opts.common_opts.create_client(None) else {
        return vec![];
    };

    let (send, mut recv) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let result = func(client, opts.common_opts).await;
        send.send(result).unwrap();
    });

    let sync_recv = thread::spawn(move || {
        let mut completions = vec![];
        while let Some(candidate) = recv.blocking_recv() {
            completions.extend(candidate);
        }
        completions
    });
    sync_recv.join().unwrap()
}

/// Returns the part of the arguments that make up CommonOpts
fn truncate_to_common_opts<I>(iter: I) -> impl Iterator<Item = String>
where
    I: IntoIterator<Item = String>,
{
    let mut it = iter.into_iter();
    it.next();
    it.next();
    let first = it.next();
    let first_it = first.map(std::iter::once).into_iter().flatten();
    let remain = it.take_while(|e| e.starts_with('-'));
    // the version command to make arg parsing successful
    let version = std::iter::once("version".to_string()).into_iter();
    first_it.chain(remain).chain(version)
}

async fn make_launcher_completion_candidate(
    client: &Client,
    launcher: &SessionLauncher,
) -> CompletionCandidate {
    let mut help = StyledStr::new();
    help.push_str(&launcher.name);
    let cc = CompletionCandidate::new(launcher.id.clone());

    let Ok(Some(project)) = client.get_project_by_id(&launcher.project_id).await else {
        return cc.help(Some(help));
    };

    help.push_str(" - ");
    help.push_str(&project.name);

    cc.help(Some(help.into()))
}

/// Complete a session launcher id
#[allow(dead_code, unused_mut, unused_variables, unreachable_code)]
pub fn complete_job_launcher_id(current: &ffi::OsStr) -> Vec<CompletionCandidate> {
    make_sync_completer2(current, async |client, _opts| {
        let Ok(launchers) = client.list_launchers().await else {
            panic!("error getting launchers");
            return vec![];
        };
        let mut result: Vec<CompletionCandidate> = vec![];
        for launcher in launchers
            .iter()
            .filter(|e| e.launcher_type == SessionMode::NonInteractive)
        {
            let cc = make_launcher_completion_candidate(&client, &launcher).await;
            result.push(cc);
        }
        return result;
    })
}
