use assert_cmd::cargo::{self, CargoError};
use snafu::Snafu;
use std::{io, process::Command};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Cargo Error: {}", source))]
    Cargo { source: CargoError },
    #[snafu(display("IO Error: {}", source))]
    IO { source: io::Error },
    #[snafu(display("JSON Error: {}", source))]
    Json { source: serde_json::Error },
}
impl std::convert::From<CargoError> for Error {
    fn from(e: CargoError) -> Self {
        Error::Cargo { source: e }
    }
}
impl std::convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO { source: e }
    }
}
impl std::convert::From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json { source: e }
    }
}

pub type Result<A> = std::result::Result<A, Error>;

pub fn mk_cmd() -> Result<Command> {
    let prg = cargo::cargo_bin!("rnk");
    let mut cmd = Command::new(prg);
    cmd.args(["--renku-url", "https://dev.renku.ch"]); //use mock url?
    Ok(cmd)
}
