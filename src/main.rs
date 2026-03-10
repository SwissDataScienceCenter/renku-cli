use rnk::error::{Error, Result};
use std::process;

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
    let opts = rnk::read_args();
    env_logger::Builder::new()
        .filter_level(opts.common_opts.verbosity.log_level_filter())
        .init();

    let result = rnk::execute_cmd(opts).await;
    result?;
    Ok(())
}

fn exit_code(err: &Error) -> i32 {
    match err {
        Error::Cmd { source: _ } => 1,
    }
}
