pub mod cli;
pub mod error;
pub mod httpclient;
pub mod util;

pub use cli::execute_cmd;

use clap::Parser;
use cli::opts::MainOpts;

/// Reads the program arguments into the `MainOpts` data structure.
pub fn read_args() -> MainOpts {
    log::debug!("Parsing command line options…");
    let m = MainOpts::parse();

    log::debug!("Parsed options: {:?}", m);
    m
}
