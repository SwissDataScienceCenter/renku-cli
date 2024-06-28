//use futures::stream::futures_unordered::FuturesUnordered;
use futures::{stream, Stream, StreamExt};
use std::{io, path::PathBuf};
use tokio::fs::{self};

// pub async fn visit_all(
//     paths: Vec<PathBuf>,
// ) -> impl Stream<Item = io::Result<DirEntry>> + Send + 'static {
//     let a = stream::iter(paths);
//     a.map(visit).flatten()
// }

// pub fn visit(
//     path: impl Into<PathBuf>,
// ) -> impl Stream<Item = io::Result<DirEntry>> + Send + 'static {
//     async fn one_level(path: PathBuf, to_visit: &mut Vec<PathBuf>) -> io::Result<Vec<DirEntry>> {
//         let mut dir = fs::read_dir(path).await?;
//         let mut files = Vec::new();

//         while let Some(child) = dir.next_entry().await? {
//             if child.metadata().await?.is_dir() {
//                 to_visit.push(child.path());
//             } else {
//                 files.push(child)
//             }
//         }

//         Ok(files)
//     }

//     stream::unfold(vec![path.into()], |mut to_visit| async {
//         let path = to_visit.pop()?;
//         let file_stream = match one_level(path, &mut to_visit).await {
//             Ok(files) => stream::iter(files).map(Ok).left_stream(),
//             Err(e) => stream::once(async { Err(e) }).right_stream(),
//         };

//         Some((file_stream, to_visit))
//     })
//     .flatten()
// }

// TODO: accept glob filter, return the root that is traversed with the path
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
