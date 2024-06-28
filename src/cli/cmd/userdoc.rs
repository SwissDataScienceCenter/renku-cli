use super::Context;
use crate::cli::sink::Error as SinkError;
use clap::{Parser, ValueHint};
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{Arena, Options};
use snafu::{ResultExt, Snafu};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;

/// Reads markdown files and processes renku-cli code blocks.
///
/// Each code block marked with `:renku-cli` is run against this
/// binary and the result is added below the command code-block.
#[derive(Parser, Debug)]
pub struct Input {
    /// The markdown files to process.
    #[arg(required = true, num_args = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
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
}

const CODE_MARKER: &str = ":renku-cli";
const RESULT_MARKER: &str = ":renku-cli-output";

impl Input {
    pub async fn exec<'a>(&self, _ctx: &Context<'a>) -> Result<(), Error> {
        let myself: PathBuf = std::env::current_exe().context(GetBinarySnafu)?;
        for file in &self.files {
            eprint!("Processing {} …\n", file.display());
            let out = process_markdown_file(file, &myself).await?;
            println!("{}", out);
        }

        Ok(())
    }
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
async fn process_markdown_file(file: &PathBuf, cli_binary: &PathBuf) -> Result<String, Error> {
    let src_md = std::fs::read_to_string(file).context(ReadFileSnafu { path: file })?;
    let src_nodes = Arena::new();
    let root = comrak::parse_document(&src_nodes, src_md.as_str(), &Options::default());
    for node in root.descendants() {
        let node_data = node.data.borrow();
        if let NodeValue::CodeBlock(ref cc) = node_data.value {
            let code_info = &cc.info;
            let command = &cc.literal;
            if code_info == CODE_MARKER {
                let cli_out = run_cli_command(cli_binary, command)?;

                let nn = src_nodes.alloc(AstNode::new(RefCell::new(Ast::new(
                    make_code_block(cli_out),
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
fn run_cli_command(cli: &PathBuf, line: &str) -> Result<String, Error> {
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
fn make_code_block(content: String) -> NodeValue {
    NodeValue::CodeBlock(NodeCodeBlock {
        fenced: true,
        fence_char: 96,
        fence_length: 3,
        fence_offset: 0,
        info: RESULT_MARKER.into(),
        literal: content,
    })
}
