use std::ffi;

use crate::{cli::opts::MainOpts, httpclient::Client};

use clap::Parser;
use clap_complete::CompletionCandidate;
use futures::executor::block_on;

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
    F: Fn(Client) -> Fut + Clone + 'static,
    Fut: Future<Output = Vec<CompletionCandidate>> + 'static,
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

    block_on(func(client))
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

/// Complete a session launcher id
#[allow(dead_code, unused_mut, unused_variables, unreachable_code)]
pub fn complete_launcher_id(current: &ffi::OsStr) -> Vec<CompletionCandidate> {
    make_sync_completer2(current, async |client| {
        let Ok(launchers) = client.list_launchers().await else {
            panic!("error getting launchers");
            return vec![];
        };
        launchers
            .iter()
            .map(|l| CompletionCandidate::new(l.id.clone()))
            .collect()
    })
    // panic!("opts: {:?}", opts);

    // let mut ulid = Ulid::from_string(&format!("test:{}", args.len())).unwrap();
    // // if copts.is_err() {
    // //     ulid = Ulid::from_string(&format!("help:{}", args.len())).unwrap()
    // // }

    // // let matches = MainOpts::command().get_matches();
    // // let rurl = matches.get_one::<RenkuUrl>("--renku-url");
    // // println!(">>>>> url: {:?}", rurl);
    // completions.push(CompletionCandidate::new(ulid.to_string()));
    // completions
}
