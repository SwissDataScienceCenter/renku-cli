//! Global error types

use crate::cli::cmd;
use color_eyre::Result as EyreResult;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    Cmd { source: cmd::CmdError },
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<cmd::CmdError> for Error {
    fn from(e: cmd::CmdError) -> Error {
        Error::Cmd { source: e }
    }
}
pub fn init() -> EyreResult<()> {
    // set up color eyre
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(true)
        .display_location_section(true)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        // use human panic in non-debug mode for more user friendly messages
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let mut metadata = metadata!();
            metadata =
                metadata.support(format!("open an issue at {}", env!("CARGO_PKG_REPOSITORY")));
            let file_path = handle_dump(&metadata, panic_info);
            // prints human-panic message
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        eprintln!("Error(hook): {}", msg);

        // use better panic to have more detailed panics in dev mode
        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));
    Ok(())
}
