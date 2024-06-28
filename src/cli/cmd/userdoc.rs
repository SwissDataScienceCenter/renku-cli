use super::Context;
use crate::cli::sink::Error as SinkError;
use clap::{Parser, ValueHint};
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{Arena, Options};
use futures::stream::TryStreamExt;
use snafu::{ResultExt, Snafu};
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Reads markdown files and processes renku-cli code blocks.
///
/// Each code block marked with `:renku-cli` is run against this
/// binary and the result is added below the command code-block.
#[derive(Parser, Debug)]
pub struct Input {
    /// The markdown file(s) to process. If a directory is given, it
    /// is traversed for `*.md` files.
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

    /// The renku-cli binary program to use for running the snippets.
    /// By default it will use itself.
    #[arg(long)]
    pub renku_cli: Option<PathBuf>,

    /// If enabled, silently overwrite existing files.
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,

    /// The code block marker to use for detecting which code blocks
    /// to extract.
    #[arg(long, default_value = ":renku-cli")]
    pub code_marker: String,

    /// The code block marker to use for annotating the result code blocks
    /// that are inserted into the document.
    #[arg(long, default_value = ":renku-cli-output")]
    pub result_marker: String,
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
}

impl Input {
    pub async fn exec<'a>(&self, _ctx: &Context<'a>) -> Result<(), Error> {
        let myself = std::env::current_exe().context(GetBinarySnafu)?;
        let bin = myself.as_path();
        let walk = crate::util::visit_all(self.files.clone()); //TODO only *.md :-)
        walk.map_err(|source| Error::ListDir { source })
            .try_for_each_concurrent(10, |entry| async move {
                eprint!("Processing {} …\n", entry.display());
                let result =
                    process_markdown_file(&entry, &bin, &self.result_marker, &self.code_marker)
                        .await?;
                match self.get_output() {
                    OutputOption::Stdout => {
                        println!("{}", result);
                    }
                    OutputOption::OutFile(f) => {
                        write_to_file(&f, &result, self.overwrite)?;
                    }
                    OutputOption::OutDir(f) => {
                        println!("write to dir {:?}", f);
                    }
                }
                Ok(())
            })
            .await?;
        Ok(())
    }
}

fn write_to_file(file: &Path, content: &str, overwrite: bool) -> Result<(), Error> {
    let mut out = std::fs::File::options()
        .write(true)
        .append(!true)
        .truncate(overwrite)
        .create(true)
        .open(file)
        .context(WriteFileSnafu)?;

    log::debug!("Write to file {:?}", out);
    out.write_all(content.as_bytes()).context(WriteFileSnafu)?;
    Ok(())
}

/// Process a markdown file by executing all included renku-cli
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
    code_marker: &str,
) -> Result<String, Error> {
    let src_md = std::fs::read_to_string(file).context(ReadFileSnafu { path: file })?;
    let src_nodes = Arena::new();
    let root = comrak::parse_document(&src_nodes, src_md.as_str(), &Options::default());
    for node in root.descendants() {
        let node_data = node.data.borrow();
        if let NodeValue::CodeBlock(ref cc) = node_data.value {
            let code_info = &cc.info;
            let command = &cc.literal;
            if code_info == code_marker {
                let cli_out = run_cli_command(cli_binary, command)?;

                let nn = src_nodes.alloc(AstNode::new(RefCell::new(Ast::new(
                    make_code_block(result_marker, cli_out),
                    node_data.sourcepos.end.clone(),
                ))));
                node.insert_after(nn);
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
    // TODO: instead of running itself as a new process, just call main
    let mut args = line.split_whitespace();
    args.next(); // skip first word which is the binary name
    let remain: Vec<&str> = args.collect();
    let cmd = Command::new(cli)
        .args(remain)
        .output()
        .context(ExecuteCliSnafu)?;
    if cmd.status.success() {
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
    NodeVa