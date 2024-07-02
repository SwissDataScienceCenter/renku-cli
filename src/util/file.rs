use futures::{stream, Stream, StreamExt};
use std::io;
use std::path::PathBuf;
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

// TODO: return root that is traversed with the path
pub fn visit_all(paths: Vec<PathBuf>) -> impl Stream<Item = io::Result<PathBuf>> + Send + 'static {
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

    stream::unfold(paths, |mut to_visit| async {
        let path = to_visit.pop()?;
        let file_stream = match one_level(path, &mut to_visit).await {
            Ok(files) => stream::iter(files).map(Ok).left_stream(),
            Err(e) => stream::once(async { Err(e) }).right_stream(),
        };

        Some((file_stream, to_visit))
    })
    .flatten()
}
