use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use clap::Parser;

use flexi_logger::Logger;

mod cli;
use cli::Cli;
use logging::LogVec;

mod logging;
mod select;
mod tui;

//TODO: select tests?

fn init_logger(logger: LogVec) {
    Logger::try_with_env_or_str("warn")
        .unwrap()
        .log_to_writer(Box::new(logger))
        .start()
        .unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logger = LogVec::new(Arc::clone(&logs));
    init_logger(logger);
    let ret = Cli::parse().exec();
    for log in logs.lock().unwrap().iter() {
        println!("{log}");
    }
    ret
}
