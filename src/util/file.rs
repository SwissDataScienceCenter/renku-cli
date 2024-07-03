use futures::TryStreamExt;
use futures::{stream, Stream, StreamExt};
use serde::Serialize;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf, StripPrefixError};
use tokio::fs::{self};

/// Puts `suffix` in the filename before the extension.
pub fn splice_name(fname: &str, suffix: &i32) -> String {
    let p = PathBuf::from(fname);

    match p.extension() {
        Some(ext) => {
            let mut base = fname.trim_end_matches(ext.to_str().unwrap()).chars();
            base.next_back();
            format!("{}_{}.{}", base.as_str(), suffix, ext.to_str().unwrap())
        }
        None => format!("{}_{}", fname, suffix),
    }
}

#[derive(Debug, Serialize)]
pub struct PathEntry {
    pub root: PathBuf,
    pub entry: PathBuf,
}

impl fmt::Display for PathEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.entry.display())
    }
}

impl PathEntry {
    pub fn sub_path(&self) -> Result<&Path, StripPrefixError> {
        self.entry.strip_prefix(&self.root)
    }
}

/// Visits all entries of the given paths recursively using tokios
/// async read_dir while returning with every entry the root element
/// it was queried for (member of the initially passed in `paths`).
pub fn visit_entries<I, P>(paths: I) -> impl Stream<Item = io::Result<PathEntry>>
where
    P: Into<PathBuf>,
    I: IntoIterator<Item = P>,
{
    //TODO figure out how to avoid heavy cloning…
    stream::iter(paths.into_iter().map(Into::into))
        .map(|p| {
            visit_all(vec![p.clone()]).map_ok(move |c| PathEntry {
                root: p.clone(),
                entry: c.clone(),
            })
        })
        .flatten()
}

/// Visits all entries of the given paths recursively using tokios async read_dir.
pub fn visit_all<I, P>(paths: I) -> impl Stream<Item = io::Result<PathBuf>> + Send + 'static
where
    P: Into<PathBuf>,
    I: IntoIterator<Item = P>,
{
    async fn one_level(path: PathBuf, to_visit: &mut Vec<PathBuf>) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if path.is_dir() {
            let mut dir = fs::read_dir(path).await?;

            while let Some(child) = dir.next_entry().await? {
                if child.metadata().await?.is_dir() {
                    to_visit.push(child.path());
                } else {
                    files.push(child.path())
                }
            }
        } else {
            files.push(path);
        }

        Ok(files)
    }

    stream::unfold(
        paths.into_iter().map(Into::into).collect::<Vec<PathBuf>>(),
        |mut to_visit| async {
            let path = to_visit.pop()?;
            let file_stream = match one_level(path, &mut to_visit).await {
                Ok(files) => stream::iter(files).map(Ok).left_stream(),
                Err(e) => stream::once(async { Err(e) }).right_stream(),
            };

            Some((file_stream, to_visit))
        },
    )
    .flatten()
}
