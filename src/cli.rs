use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "codelog",
    author,
    version,
    about = "A CLI tool to test, apply, and review patch-based Markdown logs",
    long_about = None
)]
pub struct Cli {
    #[arg(short, long, global = true, env = "CODELOG_VERBOSE", help = "Enables verbose log output")]
    pub verbose: bool,

    #[arg(long, global = true, env = "CODELOG_COLOR", value_name = "WHEN", help = "Control ANSI color output: auto, always, never")]
    pub color: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Tests applying patch codeblock(s) against a fresh empty repository in alphanumeric order")]
    Test(FileArgs),

    #[command(about = "Applies patch codeblock(s) directly to the working directory repository in alphanumeric order")]
    Apply(FileArgs),

    #[command(about = "Pretty-prints/reviews patch codeblocks from Markdown file(s) in alphanumeric order")]
    Show(FileArgs),
}

#[derive(clap::Args, Debug)]
pub struct FileArgs {
    #[arg(help = "Markdown file(s) containing patch codeblocks. Defaults to all .md files in codelog/ in alphanumeric order if empty.")]
    pub files: Vec<PathBuf>,
}
