pub mod project;
pub mod shell_completion;
#[cfg(feature = "user-doc")]
pub mod userdoc;
pub mod version;

use super::sink::{Error as SinkError, Sink};
use crate::cli::opts::{CommonOpts, ProxySetting};
use crate::httpclient::{self, proxy, Client};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

const RENKULAB_IO: &str = "https://renkulab.io";

pub struct Context {
    pub opts: CommonOpts,
    pub client: Client,
    pub renku_url: String,
}

impl Context {
    pub fn new(opts: &CommonOpts) -> Result<Context, CmdError> {
        let base_url = get_renku_url(opts);
        let client = Client::new(&base_url, proxy_settings(opts), &None, false)
            .context(ContextCreateSnafu)?;
        Ok(Context {
            opts: opts.clone(),
            client,
            renku_url: base_url,
        })
    }

    /// A short hand for `Sink::write(self.format(), value)`
    async fn write_result<A: Sink + Serialize>(&self, value: &A) -> Result<(), SinkError> {
        let fmt = self.opts.format;
        Sink::write(&fmt, value)
    }

    /// A short hand for `Sink::write_err(self.format(), value)`
    async fn write_err<A: Sink + Serialize>(&self, value: &A) -> Result<(), SinkError> {
        let fmt = self.opts.format;
        Sink::write_err(&fmt, value)
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
                RENKULAB_IO.to_string()
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

    #[cfg(feature = "user-doc")]
    #[snafu(display("UserDoc - {}", source))]
    UserDoc { source: userdoc::Error },
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

#[cfg(feature = "user-doc")]
impl From<userdoc::Error> for CmdError {
    fn from(source: userdoc::Error) -> Self {
        CmdError::UserDoc { source }
    }
}
