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
    /// The job name/id to get logs for.
    #[arg(value_hint=ValueHint::Other)]
    pub job_id: String,

    /// Periodically retrieves logs, it will stop when the job finished.
    #[arg(long, short, default_value_t = false)]
    pub follow: bool,

    /// The interval in seconds to wait between calls for logs.
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
            self.follow_logs(ctx).await.context(HttpClientSnafu)
        } else {
            self.show_logs(&ctx, 0).await.context(HttpClientSnafu)?;
            Ok(())
        }
    }

    async fn show_logs(&self, ctx: &Context, seen: usize) -> Result<usize, httpclient::Error> {
        let result = ctx.client.session_logs(&self.job_id).await?;
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

    async fn is_session_finished(&self, ctx: &Context) -> Result<bool, httpclient::Error> {
        let details = ctx.client.get_session(&self.job_id).await?;

        match &details {
            None => Ok(true),
            Some(d) => Ok(!d.status.state.is_running()),
        }
    }

    pub async fn follow_logs(&self, ctx: Context) -> Result<(), httpclient::Error> {
        let mut seen: usize = self.show_logs(&ctx, 0).await?;
        if self.is_session_finished(&ctx).await? {
            return Ok(());
        }

        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    eprintln!("Interrupted, exiting.");
                    break Ok(());
                }
                _ = sleep(Duration::from_secs(self.follow_interval as u64)) => {
                    seen = self.show_logs(&ctx, seen).await?;
                    if self.is_session_finished(&ctx).await? {
                        break Ok(());
                    }
                }
            }
        }
    }
}
