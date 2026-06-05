use crate::{
    cli::complete::complete_launcher_id,
    data::submission_id::SubmissionId,
    httpclient::{self, data::SessionStartRequest},
};

use super::Context;
use crate::cli::sink::Error as SinkError;

use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use ulid::Ulid;

use snafu::{ResultExt, Snafu};

/// Start a job.
///
/// Starts a non-interactive session using a pre-configured session launcher.
#[derive(Parser, Debug)]
pub struct Input {
    /// The launcher to use for launching the job.
    #[arg(long, value_hint=ValueHint::Other, add = ArgValueCompleter::new(complete_launcher_id))]
    pub launcher: Ulid,

    /// A submission id allows to deduplicate same job submissions. If missing, a random one is generated. It must be at least 4 characters, starting with a lowercase letter, followed by alphanumeric characters (including the dash).
    #[arg(long)]
    pub submission_id: Option<SubmissionId>,

    /// These arguments are passed to the renku job command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0.., value_name = "ARGS")]
    pub passthrough: Vec<String>,
}

// #[allow(dead_code, unused_mut, unused_variables, unreachable_code)]
// fn complete_launcher_id(current: &ffi::OsStr) -> Vec<CompletionCandidate> {
//     let mut completions = vec![];
//     let Some(_current) = current.to_str() else {
//         return completions;
//     };

//     let mut args = std::env::args()
//         .take_while(|e| !e.eq_ignore_ascii_case("job"))
//         .skip(2)
//         .collect::<Vec<String>>();
//     args.push("version".into());
//     let Ok(opts) = MainOpts::try_parse_from(args) else {
//         return completions;
//     };

//     let Ok(client) = opts.common_opts.create_client(None) else {
//         return completions;
//     };

//     // let args2 = args.take_while(|e| !e.eq_ignore_ascii_case("job")).skip(3);
//     // let args3: Vec<String> = args2.collect();

//     // let mut def_opts = CommonOpts::empty();
//     // def_opts.try_update_from(args2).unwrap();

//     panic!("opts: {:?}", opts);

//     let mut ulid = Ulid::from_string(&format!("test:{}", args.len())).unwrap();
//     // if copts.is_err() {
//     //     ulid = Ulid::from_string(&format!("help:{}", args.len())).unwrap()
//     // }

//     // let matches = MainOpts::command().get_matches();
//     // let rurl = matches.get_one::<RenkuUrl>("--renku-url");
//     // println!(">>>>> url: {:?}", rurl);
//     completions.push(CompletionCandidate::new(ulid.to_string()));
//     completions
// }

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let submission_id = self
            .submission_id
            .clone()
            .unwrap_or_else(|| SubmissionId::random());
        let req = SessionStartRequest {
            launcher_id: self.launcher.to_string(),
            session_type: "non-interactive".into(),
            submission_id: Some(submission_id),
            job_args_override: self.passthrough.clone(),
        };
        let result = ctx
            .client
            .start_session(req)
            .await
            .context(HttpClientSnafu)?;

        ctx.write_result(&result).await.context(WriteResultSnafu)
    }
}
