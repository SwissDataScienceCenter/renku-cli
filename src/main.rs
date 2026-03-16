use clap::CommandFactory;
use clap_complete::CompleteEnv;
use color_eyre::Result as EyreResult;
use rnk::cli::opts::MainOpts;
use rnk::error::Result;

#[tokio::main]
async fn main() -> EyreResult<()> {
    rnk::error::init()?;
    execute().await?;
    Ok(())
}

async fn execute() -> Result<()> {
    CompleteEnv::with_factory(MainOpts::command).complete();
    let opts = rnk::read_args();
    env_logger::Builder::new()
        .filter_level(opts.common_opts.verbosity.log_level_filter())
        .init();

    let result = rnk::execute_cmd(opts).await;
    result?;
    Ok(())
}
