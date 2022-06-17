use std::{error::Error, path::Path};

use clap::{Args, Parser, Subcommand};
use fuzzy_matcher::skim::SkimMatcherV2;

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

use crate::select::{
    new_complete_manifest_from_path, print_targets, score_targets, targets_from_manifest,
};

impl Cli {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        let Commands::SelectCommand(command) = self.command;
        let manifest = new_complete_manifest_from_path(Path::new("."))?;
        let targets = targets_from_manifest(&manifest, Path::new("."));
        let skim = SkimMatcherV2::default();
        match command.pattern {
            Some(pattern) => {
                let scored_targets = score_targets(&targets, &pattern, &skim);
                match command.cargo_command.as_deref() {
                    Some("run") | Some("r") => {
                        let (target, _score) = scored_targets.last().ok_or("No targets")?;
                        log::info!("Selected target: {} ({})", target.name, target.target_type);
                        println!("Selected target: {} ({})", target.name, target.target_type);
                        log::debug!("Creating cargo command.");
                        let mut proc_command = std::process::Command::new("cargo");
                        proc_command
                            .arg("run")
                            .arg(target.target_type.to_cargo_flag())
                            .arg(&target.name)
                            .args(command.cargo_args);

                        log::info!("Spawning cargo command: {proc_command:?}");
                        proc_command.spawn()?.wait()?;
                    }
                    _ => {
                        log::info!("No command provided, printing out scored targets");
                        for (target, _score) in &scored_targets {
                            println!("{}", target);
                        }
                    }
                }
            }
            None => print_targets(&targets),
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(name = "select")]
    SelectCommand(SelectCommand),
}

#[derive(Args, Debug)]
pub struct SelectCommand {
    #[clap(value_parser)]
    pub pattern: Option<String>,
    #[clap(value_parser)]
    pub cargo_command: Option<String>,
    // TODO: make it a "last" argument, i.e. after "--"
    #[clap(value_parser)]
    pub cargo_args: Vec<String>,
}
