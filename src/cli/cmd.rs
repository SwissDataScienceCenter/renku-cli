pub mod dataset;
pub mod job;
pub mod login;
pub mod project;
pub mod update;
#[cfg(feature = "user-doc")]
pub mod userdoc;
pub mod version;

use super::sink::{Error as SinkError, Sink};
use crate::cli::opts::CommonOpts;
use crate::data::renku_url::RenkuUrl;
use crate::httpclient::{self, Client};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

pub struct Context {
    pub opts: CommonOpts,
    pub client: Client,
}

impl Context {
    pub fn new(opts: &CommonOpts) -> Result<Context, CmdError> {
        let client = opts.create_client(None).context(ContextCreateSnafu)?;
        Ok(Context {
            opts: opts.clone(),
            client,
        })
    }

    pub fn renku_url(&self) -> &RenkuUrl {
        self.client.base_url()
    }

    /// A short hand for `Sink::write_out(self.format(), value)`
    async fn write_result<A: Sink + Serialize>(&self, value: &A) -> Result<(), SinkError> {
        let fmt = self.opts.format;
        Sink::write_out(&fmt, value)
    }

    /// A short hand for `Sink::write_err(self.format(), value)`
    async fn write_err<A: Sink + Serialize>(&self, value: &A) -> Result<(), SinkError> {
        let fmt = self.opts.format;
        Sink::write_err(&fmt, value)
    }
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("ContextCreate - {}", source))]
    ContextCreate { source: httpclient::Error },

    #[snafu(display("Version - {}", source))]
    Version { source: version::Error },

    #[snafu(display("Update - {}", source))]
    Update { source: update::Error },

    #[snafu(display("Project - {}", source))]
    Project { source: project::Error },

    #[snafu(display("Login - {}", source))]
    Login { source: login::Error },

    #[cfg(feature = "user-doc")]
    #[snafu(display("UserDoc - {}", source))]
    UserDoc { source: userdoc::Error },

    #[snafu(display("Dataset - {}", source))]
    Dataset { source: dataset::Error },

    #[snafu(display("Job - {}", source))]
    Job { source: job::Error },
}

impl From<job::Error> for CmdError {
    fn from(source: job::Error) -> Self {
        CmdError::Job { source }
    }
}

impl From<version::Error> for CmdError {
    fn from(source: version::Error) -> Self {
        CmdError::Version { source }
    }
}
impl From<update::Error> for CmdError {
    fn from(source: update::Error) -> Self {
        CmdError::Update { source }
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

impl From<login::Error> for CmdError {
    fn from(source: login::Error) -> Self {
        CmdError::Login { source }
    }
}

impl From<dataset::Error> for CmdError {
    fn from(source: dataset::Error) -> Self {
        CmdError::Dataset { source }
    }
}
