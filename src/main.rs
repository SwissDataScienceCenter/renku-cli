use renku_cli::error::{Error, Result};
use std::env;
use std::process;

const LOG_LEVEL: &str = "RUST_LOG";

#[tokio::main]
async fn main() {
    let error_style = console::Style::new().red().bright();
    let result = execute().await;
    if let Err(err) = result {
        eprintln!("{}", error_style.apply_to(&err));
        process::exit(exit_code(&err));
    }
}

async fn execute() -> Result<()> {
    let opts = renku_cli::read_args();
    let remove_env = match opts.common_opts.verbose {
        1 => set_log_level("info"),
        n => {
            if n > 1 {
                set_log_level("debug")
            } else {
                false
            }
        }
    };
    env_logger::init();

    let result = renku_cli::execute_cmd(opts).await;
    if remove_env {
        env::remove_var(LOG_LEVEL);
    }
    result?;
    Ok(())
}

fn set_log_level(level: &str) -> bool {
    let current = env::var_os(LOG_LEVEL);
    if current.is_none() {
        env::set_var(LOG_LEVEL, level);
        true
    } else {
        false
    }
}

fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Cmd { source: _ } => 1,
    }
}
