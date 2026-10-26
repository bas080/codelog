use codelog::cli::Cli;
use clap::Parser;
use std::process::exit;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = codelog::app::run(cli) {
        eprintln!("{}", err);
        exit(1);
    }
}
