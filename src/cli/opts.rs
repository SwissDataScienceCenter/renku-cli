use super::cmd::*;
use clap::{ArgAction, Parser, ValueEnum, ValueHint};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Main options are available to all commands. They must appear
/// before a sub-command.
#[derive(Parser, Debug, Clone)]
#[command()]
pub struct CommonOpts {
    /// Be more verbose when logging. Verbosity increases with each
    /// occurence of that option.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// How to format the output. The default is human readable which
    /// may choose to not show every detail for better readability.
    /// The json output format can be used to always show all details
    /// in a structured form.
    #[arg(short, long, value_enum, default_value_t = Format::Default)]
    pub format: Format,

    /// The (base) URL to Renku. It can be given as environment
    /// variable RENKU_CLI_RENKU_URL.
    #[arg(long, value_hint = ValueHint::Url)]
    pub renku_url: Option<Url>,

    /// Set a proxy to use for doing http requests. By default, the
    /// system proxy will be used. Can be either `none` or <url>. If
    /// `none`, the system proxy will be ignored; otherwise specify
    /// the proxy url, like `http://myproxy.com`.
    #[arg(long)]
    pub proxy: Option<ProxySetting>,

    /// The user to authenticate at the proxy.
    #[arg(long)]
    pub proxy_user: Option<String>,

    /// The password to authenticate at the proxy.
    #[arg(long)]
    pub proxy_password: Option<String>,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command()]
    Version(version::Input),

    #[command()]
    ShellCompletion(shell_completion::Input),

    #[command()]
    Project(project::Input),

    #[cfg(feature = "user-doc")]
    UserDoc(userdoc::Input),
}

/// This is the command line interface to the Renku platform. Main
/// options are available to all sub-commands and must appear before
/// them. Each sub command has its own set of flags/options and
/// arguments.
///
/// Repository: https://github.com/SwissDataScienceCenter/renku-cli
/// Issue tracker: https://github.com/SwissDataScienceCenter/renku-cli/issues
#[derive(Parser, Debug)]
#[command(name = "rnk", version)]
pub struct MainOpts {
    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// The format for presenting the results.
#[derive(ValueEnum, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Format {
    Json,
    Default,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub enum ProxySetting {
    /// Don't use any proxy; this will also discard the system proxy.
    None,

    /// Use a custom defined proxy.
    Custom { url: String },
}

impl FromStr for ProxySetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            Ok(ProxySetting::None)
        } else {
            Ok(ProxySetting::Custom { url: s.to_string() })
        }
    }
}
