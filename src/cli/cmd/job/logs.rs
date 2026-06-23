use super::Context;
use crate::{cli::sink::Error as SinkError, httpclient};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;

use clap::{Parser, ValueHint};

use snafu::{ResultExt, Snafu};

/// Listing logs of a jobs.
///
/// List the logs of a job.
#[derive(Parser, Debug)]
pub struct Input {
    /// The job name/id to get logs for
    #[arg(value_hint=ValueHint::Other)]
    pub job_id: String,

    /// Periodically retrieves logs
    #[arg(long, short, default_value_t = false)]
    pub follow: bool,

    /// The interval in seconds to wait between calls for logs
    #[arg(long, default_value_t = 2)]
    pub follow_interval: u8,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Http error: {}", source))]
    HttpClient { source: httpclient::Error },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        if self.follow {
            self.follow_logs(ctx).await
        } else {
            self.show_logs(&ctx, 0).await?;
            Ok(())
        }
    }

    async fn show_logs(&self, ctx: &Context, seen: usize) -> Result<usize, Error> {
        let result = ctx
            .client
            .session_logs(&self.job_id)
            .await
            .context(HttpClientSnafu)?;
        if let Some(lines_blob) = result.0.get("amalthea-session") {
            let lines: Vec<&str> = lines_blob.lines().collect();
            if lines.len() > seen {
                for line in &lines[seen..] {
                    println!("{}", line);
                }
                return Ok(lines.len());
            }
        }
        Ok(seen)
    }

    async fn follow_logs(&self, ctx: Context) -> Result<(), Error> {
        let mut seen: usize = self.show_logs(&ctx, 0).await?;
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    eprintln!("Interrupted, exiting.");
                    break Ok(());
                }
                _ = sleep(Duration::from_secs(self.follow_interval as u64)) => {
                    seen = self.show_logs(&ctx, seen).await?;
                }
            }
        }
    }
}
