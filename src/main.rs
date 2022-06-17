use std::error::Error;

use clap::Parser;

use flexi_logger::Logger;

mod cli;
use cli::Cli;

mod select;

//TODO: TUI like fzf
//TODO: select tests?

fn init_logger() {
    Logger::try_with_env_or_str("warn")
        .unwrap()
        .start()
        .unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    Cli::parse().exec()
}
