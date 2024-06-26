pub mod project;
pub mod shell_completion;
pub mod version;

use super::sink::{Error as SinkError, Sink};
use crate::cli::opts::{CommonOpts, Format, ProxySetting};
use crate::httpclient::{self, proxy, Client};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

pub trait Cmd {
    type CmdError;

    fn exec(&self, args: &Context) -> Result<(), Self::CmdError>;
}

pub struct Context<'a> {
    pub opts: &'a CommonOpts,
    pub client: Client,
    pub renku_url: String,
}

const RENKULABIO: &str = "https://renkulab.io";

impl Context<'_> {
    pub fn new(opts: &CommonOpts) -> Result<Context, CmdError> {
        let base_url = get_renku_url(opts);
        let client = Client::new(&base_url, proxy_settings(opts), &None, false)
            .context(ContextCreateSnafu)?;
        Ok(Context {
            opts,
            client,
            renku_url: base_url,
        })
    }

    /// A short hand for `Sink::write(self.format(), value)`
    fn write_result<A: Sink + Serialize>(&self, value: A) -> Result<(), SinkError> {
        let fmt = self.format();
        Sink::write(fmt, &value)
    }

    fn format(&self) -> Format {
        self.opts.format.unwrap_or(Format::Default)
    }
}

fn get_renku_url(opts: &CommonOpts) -> String {
    match &opts.renku_url {
        Some(u) => {
            log::debug!("Use renku url from arguments: {}", u);
            u.clone()
        }
        None => match std::env::var("RENKU_CLI_RENKU_URL").ok() {
            Some(u) => {
                log::debug!("Use renku url from env RENKU_CLI_RENKU_URL: {}", u);
                u
            }
            None => {
                log::debug!("Use renku url: https://renkulab.io");
                RENKULABIO.to_string()
            }
        },
    }
}

fn proxy_settings(opts: &CommonOpts) -> proxy::ProxySetting {
    let user = opts.proxy_user.clone();
    let password = opts.proxy_password.clone();
    let prx = opts.proxy.clone();

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

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("ContextCreate - {}", source))]
    ContextCreate { source: httpclient::Error },

    #[snafu(display("Version - {}", source))]
    Version { source: version::Error },

    #[snafu(display("Project - {}", source))]
    Project { source: project::Error },
}

impl From<version::Error> for CmdError {
    fn from(source: version::Error) -> Self {
        CmdError::Version { source }
    }
}

impl From<project::Error> for CmdError {
    fn from(source: project::Error) -> Self {
        CmdError::Project { source }
    }
}
