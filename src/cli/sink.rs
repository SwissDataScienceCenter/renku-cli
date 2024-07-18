use crate::cli::opts::Format;
use crate::data::simple_message::SimpleMessage;
use crate::httpclient::data::*;
use crate::util::file::PathEntry;
use serde::Serialize;
use snafu::Snafu;
use std::fmt::Display;

use super::BuildInfo;

#[derive(Debug, Snafu)]
pub enum Error {
    Json { source: serde_json::Error },
}

pub trait Sink
where
    Self: Serialize + Display,
{
    fn write(format: &Format, value: &Self) -> Result<(), Error> {
        match format {
            Format::Json => {
                serde_json::to_writer(std::io::stdout(), value)?;
                Ok(())
            }
            Format::Default => {
                println!("{}", value);
                Ok(())
            }
        }
    }
    fn write_err(format: &Format, value: &Self) -> Result<(), Error> {
        match format {
            Format::Json => {
                serde_json::to_writer(std::io::stderr(), value)?;
                Ok(())
            }
            Format::Default => {
                eprintln!("{}", value);
                Ok(())
            }
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json { source: e }
    }
}

impl Sink for ProjectDetails {}
impl Sink for SimpleMessage {}
impl Sink for BuildInfo {}
impl Sink for PathEntry {}
