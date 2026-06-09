use crate::{
    data::{
        project_id::{ProjectId, ProjectIdParseError},
        renku_url::RenkuUrl,
    },
    httpclient::{Client, Error as ClientError, proxy},
};

use super::cmd::*;
use clap::{Parser, ValueEnum, ValueHint};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

/// Main options are available to all commands. They must appear
/// before a sub-command.
#[derive(Parser, Debug, Clone)]
#[command()]
pub struct CommonOpts {
    /// Be more verbose when logging. Verbosity increases with each
    /// occurence of that option.
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// How to format the output. The default is human readable which
    /// may choose to not show every detail for better readability.
    /// The json output format can be used to always show all details
    /// in a structured form.
    #[arg(short, long, value_enum, default_value_t = Format::Default)]
    pub format: Format,

    /// The (base) URL to Renku. It can be given as environment
    /// variable RENKU_CLI_RENKU_URL.
    #[arg(long, value_hint = ValueHint::Url)]
    pub renku_url: Option<RenkuUrl>,

    /// Some commands may operate within a project. If this option is
    /// set, or the environment variable RENKU_CLI_PROJECT_CONTEXT is
    /// present and specifies a project id, some commands use it to
    /// confine there functionality to this project.
    #[arg(long, value_hint = ValueHint::Url)]
    pub project_context: Option<ProjectId>,

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

impl CommonOpts {
    const ACCESS_TOKEN_ENV: &str = "RENKU_CLI_ACCESS_TOKEN";

    pub fn create_client(&self, trusted_cert: Option<PathBuf>) -> Result<Client, ClientError> {
        let at = std::env::var(Self::ACCESS_TOKEN_ENV).ok();
        let base_url = self
            .get_renku_url()
            .map_err(|e| ClientError::UrlParse { source: e })?;
        Client::new(base_url, self.proxy_settings(), trusted_cert, false, at)
    }

    fn proxy_settings(&self) -> proxy::ProxySetting {
        let user = self.proxy_user.clone();
        let password = self.proxy_password.clone();
        let prx = self.proxy.clone();

        log::debug!("Using proxy: {:?} @ {:?}", user, prx);
        match prx {
            None => proxy::ProxySetting::System,
            Some(ProxySetting::None) => proxy::ProxySetting::None,
            Some(ProxySetting::Custom { url }) => proxy::ProxySetting::Custom {
                url: url.clone(),
                user,
                password,
            },
        }
    }

    fn get_renku_url(&self) -> Result<RenkuUrl, url::ParseError> {
        match &self.renku_url {
            Some(u) => {
                log::debug!("Use renku url from arguments: {}", u);
                Ok(u.clone())
            }
            None => match RenkuUrl::from_env() {
                Some(res) => {
                    if let Ok(u) = &res {
                        log::debug!("Use renku url from env RENKU_CLI_RENKU_URL: {}", u);
                    }
                    res
                }
                None => {
                    log::debug!("Use renku url: https://renkulab.io");
                    Ok(RenkuUrl::renkulab_io())
                }
            },
        }
    }

    #[allow(dead_code)]
    fn get_project_context(&self) -> Result<Option<ProjectId>, ProjectIdParseError> {
        if self.project_context.is_some() {
            return Ok(self.project_context.clone());
        } else {
            match std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok() {
                Some(id) => ProjectId::parse(&id).map(|e| Some(e)),
                None => Ok(None),
            }
        }
    }
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command()]
    Version(version::Input),
    #[command()]
    Update(update::Input),

    #[command()]
    Project(project::Input),

    /// Clone a project. (Shortcut for 'project clone')
    #[command()]
    Clone(project::clone::Input),

    #[command()]
    Login(login::Input),

    #[cfg(feature = "user-doc")]
    UserDoc(userdoc::Input),

    #[command()]
    Dataset(dataset::Input),

    #[command()]
    Job(job::Input),
}

/// This is the command line interface to the Renku platform. Main
/// options are available to all sub-commands and must appear before
/// them. Each sub command has its own set of flags/options and
/// arguments.
///
/// Repository: <https://github.com/SwissDataScienceCenter/renku-cli>
/// Issue tracker: <https://github.com/SwissDataScienceCenter/renku-cli/issues>
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
