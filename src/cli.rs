pub mod cmd;
pub mod opts;
pub mod sink;

use self::cmd::{Cmd, CmdError, Context};
use self::opts::{MainOpts, SubCommand};
use clap::CommandFactory;
use serde::Serialize;
use std::fmt;

pub fn execute_cmd(opts: MainOpts) -> Result<(), CmdError> {
    let ctx = Context::new(&opts.common_opts)?;

    log::info!("Running command: {:?}", opts.subcmd);
    match &opts.subcmd {
        SubCommand::Version(input) => input.exec(&ctx)?,
        SubCommand::ShellCompletion(input) => {
            let mut app = MainOpts::command();
            input.print_completions(&mut app);
        }
        SubCommand::Project(input) => input.exec(&ctx)?,
    };
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct BuildInfo {
    pub build_date: &'static str,
    pub build_version: &'static str,
    pub git_commit: &'static str,
    pub rustc_host_triple: &'static str,
    pub rustc_llvm_version: &'static str,
    pub rustc_version: &'static str,
    pub cargo_target_triple: &'static str,
}
impl Default for BuildInfo {
    fn default() -> Self {
        BuildInfo {
            build_date: env!("VERGEN_BUILD_TIMESTAMP"),
            build_version: env!("CARGO_PKG_VERSION"),
            git_commit: env!("VERGEN_GIT_SHA"),
            rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
            rustc_llvm_version: env!("VERGEN_RUSTC_LLVM_VERSION"),
            rustc_version: env!("VERGEN_RUSTC_SEMVER"),
            cargo_target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
        }
    }
}
impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cc = &self.git_commit[..8];
        write!(
            f,
            "  Built at: {}\n  Version: {}\n  Sha: {}",
            self.build_date, self.build_version, cc
        )
    }
}
