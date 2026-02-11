use super::zenodo;
use super::Context;
use crate::httpclient::Error as HttpError;
use clap::Parser;
use snafu::{ResultExt, Snafu};
use url::Url;

enum Provider {
    Zenodo,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A dataset deposit error occured: {}", source))]
    HttpClient { source: HttpError },
}

/// Copies the data from a location into a data deposit
#[derive(Parser, Debug)]
pub struct CopyInput {
    /// The id of the deposit the data should be copied to.
    #[arg()]
    pub deposit_id: String,

    /// The provider for the dataset
    #[arg()]
    pub provider: Provider,

    /// The source directory where the files to be copied can be found.
    #[arg()]
    pub source_dir: PathBuf,
}

impl CopyInput {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        match self.provider {
            Provider::Zenodo => {
                let token = std::env::var("ZENODO_API_KEY")?;
                let clnt = zenodo::ZenodoClient::new(token);
                clnt.upload_files(self.deposit_id, self.source_dir).await?;
            }
        }
    }
}
