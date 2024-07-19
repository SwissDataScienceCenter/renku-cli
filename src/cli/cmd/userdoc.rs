use super::Context;
use crate::cli::opts::Format;
use crate::cli::sink::{Error as SinkError, Sink};
use crate::util::file as file_util;
use crate::util::file::PathEntry;
use clap::{Parser, ValueHint};
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{Arena, Options};
use futures::future;
use futures::stream::TryStreamExt;
use regex::Regex;
use serde::Serialize;
use snafu::{ResultExt, Snafu};
use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Reads markdown files and processes renku-cli code blocks.
///
/// Each code block marked with `renku-cli` or `rnk` is run against
/// this binary and the result is added below the command code-block.
///
/// If you use `renku-cli:silent` or `rnk:silent` the command will be
/// run, but the output is ignored.
#[derive(Parser, Debug)]
pub struct Input {
    /// The markdown file(s) to process. If a directory is given, it
    /// is traversed for `*.md` files by default. Use
    /// `--filter-regex` to change this filter.
    #[arg(required = true, num_args = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,

    /// Write the output into this file. If multiple files are
    /// processed the output is appended into this single file. Either
    /// `--output-file` or `--output-dir` can be used.
    #[arg(long, group = "output")]
    pub output_file: Option<PathBuf>,

    /// Write the output files into this directory. The file names are
    /// used from the input files. If a directory is traversed for
    /// markdown files, the sub-dirs are recreated in the target
    /// directory. Either `--output-file` or `--output-dir` can be
    /// used.
    #[arg(long, group = "output")]
    pub output_dir: Option<PathBuf>,

    /// The rnk binary program to use for running the snippets. By
    /// default it will use itself.
    #[arg(long)]
    pub renku_cli: Option<PathBuf>,

    /// If enabled, silently overwrite existing files.
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,

    /// The code block marker to use for annotating the result code blocks
    /// that are inserted into the document.
    #[arg(long, default_value = "renku-cli-output")]
    pub result_marker: String,

    /// A regex for filtering files when traversing directories. By
    /// default only markdown (*.md) files are picked up. The regex is
    /// matched against the simple file name (not the absolute one
    /// including the full path).
    #[arg(long, default_value = "^.*\\.md$")]
    pub filter_regex: Regex,
}

enum OutputOption<'a> {
    OutFile(&'a Path),
    OutDir(&'a Path),
    Stdout,
}
impl Input {
    fn get_output(&self) -> OutputOption {
        if let Some(f) = &self.output_file {
            OutputOption::OutFile(f.as_path())
        } else if let Some(f) = &self.output_dir {
            OutputOption::OutDir(f.as_path())
        } else {
            OutputOption::Stdout
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Cannot get renku cli binary: {}", source))]
    GetBinary { source: std::io::Error },

    #[snafu(display("Cannot read file '{}': {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Error listing files: {}", source))]
    ListDir { source: std::io::Error },

    #[snafu(display("Error creating directory: {}", source))]
    CreateDir { source: std::io::Error },

    #[snafu(display("Error writing file: {}", source))]
    WriteFile { source: std::io::Error },

    #[snafu(display("Cannot format to common mark: {}", source))]
    CommonMarkFormat { source: std::io::Error },

    #[snafu(display("Cannot convert bytes to string: {}", source))]
    Utf8Decode { source: std::string::FromUtf8Error },

    #[snafu(display("Error running cli: {}", source))]
    ExecuteCli { source: std::io::Error },

    #[snafu(display("Cli returned not successful: {}: {}", status, stderr))]
    CliResult {
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[snafu(display("The file already exists: {}", file.display()))]
    ExistingOutput { file: PathBuf },

    #[snafu(display("Error rebasing target path: {}", source))]
    PathPrefix { source: std::path::StripPrefixError },
}

impl Input {
    pub async fn exec(&self, ctx: Context) -> Result<(), Error> {
        let md_regex: &Regex = &self.filter_regex;
        let myself = std::env::current_exe().context(GetBinarySnafu)?;
        let bin = match &self.renku_cli {
            Some(p) => p.as_path(),
            None => myself.as_path(),
        };

        let fmt = ctx.opts.format;
        let walk = file_util::visit_entries(self.files.iter())
            .try_filter(|p| future::ready(Self::path_match(&p.entry, md_regex)));
        walk.map_err(|source| Error::ListDir { source })
            .try_for_each_concurrent(10, |entry| async move {
                let result = process_markdown_file(&entry.entry, bin, &self.result_marker).await?;
                match self.get_output() {
                    OutputOption::Stdout => {
                        if fmt != Format::Json {
                            println!("{}", result);
                        }
                    }
                    OutputOption::OutFile(f) => {
                        write_to_file(f, &result, self.overwrite, true)?;
                    }
                    OutputOption::OutDir(f) => {
                        write_to_dir(&entry, f, &result, self.overwrite)?;
                    }
                }
                let res = Processed {
                    entry,
                    output: result,
                };
                Sink::write_out(&fmt, &res).context(WriteResultSnafu)?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    fn path_match(p: &Path, regex: &Regex) -> bool {
        match p.file_name().and_then(|n| n.to_str()) {
            Some(name) => regex.is_match(name),
            None => false,
        }
    }
}

fn write_to_dir(
    entry: &PathEntry,
    target: &Path,
    content: &str,
    overwrite: bool,
) -> Result<(), Error> {
    let out = target.join(entry.sub_path().context(PathPrefixSnafu)?);
    log::debug!("Writing {} docs to {}", entry, out.display());
    if let Some(p) = out.parent() {
        log::debug!("Ensuring directory: {}", p.display());
        std::fs::create_dir_all(p).context(CreateDirSnafu)?
    }
    write_to_file(&out, content, overwrite, false)?;
    Ok(())
}

fn write_to_file(file: &Path, content: &str, overwrite: bool, append: bool) -> Result<(), Error> {
    let mut out = std::fs::File::options()
        .write(true)
        .truncate(overwrite && !append)
        .create(overwrite)
        .create_new(!overwrite)
        .append(append)
        .open(file)
        .context(WriteFileSnafu)?;

    log::debug!("Write to file {:?}", out);
    out.write_all(content.as_bytes()).context(WriteFileSnafu)?;
    Ok(())
}

/// Process a markdown file by executing all included rnk
/// commands and inserting the results.
///
/// The commands are taken from (fenced) code blocks marked with
/// `:renku-cli`. The first word in that command is replaced with the
/// configured binary and the remainder is passed as is.
///
/// It returns common-mark string with the results of the cli
/// included.
async fn process_markdown_file(
    file: &PathBuf,
    cli_binary: &Path,
    result_marker: &str,
) -> Result<String, Error> {
    let src_md = std::fs::read_to_string(file).context(ReadFileSnafu { path: file })?;
    let src_nodes = Arena::new();
    let root = comrak::parse_document(&src_nodes, src_md.as_str(), &Options::default());
    for node in root.descendants() {
        let node_data = node.data.borrow();
        if let NodeValue::CodeBlock(ref cc) = node_data.value {
            let command = &cc.literal;
            log::debug!("Process code block: {}", &cc.info);
            match parse_fence_info(&cc.info) {
                None => {
                    log::debug!("Code block not processed: {}", &cc.info);
                }
                Some(FenceModifier::Default) => {
                    log::debug!("Run code block and insert result for: {}", &cc.info);
                    let cli_out = run_cli_command(cli_binary, command)?;
                    let nn = src_nodes.alloc(AstNode::new(RefCell::new(Ast::new(
                        make_code_block(result_marker, cli_out),
                        node_data.sourcepos.end,
                    ))));
                    node.insert_after(nn);
                }
                Some(FenceModifier::Silent) => {
                    log::debug!("Run code block and ignore result for: {}", &cc.info);
                    run_cli_command(cli_binary, command)?;
                }
            }
        }
    }
    let mut out_md = vec![];
    comrak::format_commonmark(root, &Options::default(), &mut out_md)
        .context(CommonMarkFormatSnafu)?;
    String::from_utf8(out_md).context(Utf8DecodeSnafu)
}

/// Run the given command line using the given binary.
fn run_cli_command(cli: &Path, line: &str) -> Result<String, Error> {
    log::debug!("Run: {} {}", cli.display(), line);
    // TODO: instead of running itself as a new process, just call main
    let mut args = line.split_whitespace();
    args.next(); // skip first word which is the binary name
    let remain: Vec<&str> = args.collect();
    // TODO use tokio::process instead
    let cmd = Command::new(cli)
        .args(remain)
        .output()
        .context(ExecuteCliSnafu)?;
    if cmd.status.success() {
        log::debug!("Command ran successful");
        let out = String::from_utf8(cmd.stdout).context(Utf8DecodeSnafu)?;
        Ok(out)
    } else {
        let err = String::from_utf8(cmd.stderr).context(Utf8DecodeSnafu)?;
        Err(Error::CliResult {
            status: cmd.status,
            stderr: err,
        })
    }
}

/// Wraps a string into a fenced code block
fn make_code_block(marker: &str, content: String) -> NodeValue {
    NodeValue::CodeBlock(NodeCodeBlock {
        fenced: true,
        fence_char: 96,
        fence_length: 3,
        fence_offset: 0,
        info: marker.into(),
        literal: content,
    })
}

enum FenceModifier {
    Default,
    Silent,
}

impl FromStr for FenceModifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "renku-cli" => Ok(FenceModifier::Default),
            "rnk" => Ok(FenceModifier::Default),
            "renku-cli:silent" => Ok(FenceModifier::Silent),
            "rnk:silent" => Ok(FenceModifier::Silent),
            &_ => Err(format!("Invalid modifier: {}", s)),
        }
    }
}

fn parse_fence_info(info: &str) -> Option<FenceModifier> {
    log::debug!("Read fence info: {}", info);
    let mut parts = info.split_whitespace();
    parts.next(); // skip language definition
    parts.next().and_then(|s| FenceModifier::from_str(s).ok())
}

#[derive(Debug, Serialize)]
struct Processed {
    pub entry: PathEntry,
    pub output: String,
}

impl fmt::Display for Processed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processed {} ...", self.entry.entry.display())
    }
}

impl Sink for Processed {}
