use crate::cli::opts::Format;
use serde::Serialize;
use snafu::Snafu;
use std::fmt::Display;

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
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json { source: e }
    }
}
