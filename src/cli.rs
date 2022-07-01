use std::{error::Error, path::Path};

use clap::{Args, Parser, Subcommand};
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::tui::Tui;

use crate::select::{
    new_complete_manifest_from_path, score_targets, targets_from_manifest, Target,
};

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo", version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(name = "select")]
    SelectCommand(SelectCommand),
}

/// Fuzzy-match targets/examples
#[derive(Args, Debug)]
#[clap(version, author)]
pub struct SelectCommand {
    #[clap(
        value_parser,
        help = "Cargo command to run with selected target (e.g. \"run\")."
    )]
    pub cargo_command: Option<String>,
    #[clap(
        value_parser,
        help = "Pattern to fuzzy-match targets with. Omit for interactive mode."
    )]
    pub pattern: Option<String>,
    #[clap(value_parser, help = "Additional arguments to pass to cargo.")]
    pub cargo_args: Vec<String>,
}
impl Cli {
    pub fn exec(mut self) -> Result<(), Box<dyn Error>> {
        let Commands::SelectCommand(ref mut command) = self.command;
        let manifest = new_complete_manifest_from_path(Path::new("."))?;
        let targets = targets_from_manifest(&manifest, Path::new("."));
        let pattern = match command.pattern.take() {
            Some(pattern) => pattern,
            None => Tui::launch(&targets)?,
        };
        self.do_stuff_with_targets(&targets, &pattern)
    }

    fn do_stuff_with_targets(
        &self,
        targets: &[Target],
        pattern: &str,
    ) -> Result<(), Box<dyn Error>> {
        let Commands::SelectCommand(command) = &self.command;
        let scored_targets = score_targets(targets, pattern, &SkimMatcherV2::default());
        match command.cargo_command.as_deref() {
            Some("run") | Some("r") => {
                let target = scored_targets.last().ok_or("No targets")?;
                log::info!("Selected target: {target}.");
                println!("Selected target: {} ({})", target.name, target.target_type);
                log::debug!("Creating cargo command.");
                let mut proc_command = std::process::Command::new("cargo");
                proc_command
                    .current_dir(&target.workspace_path)
                    .arg("run")
                    .arg(target.target_type.to_cargo_flag())
                    .arg(&target.name)
                    .args(&command.cargo_args);

                log::info!(
                    "Spawning cargo command: {proc_command:?} in {:#?}",
                    target.workspace_path
                );
                proc_command.spawn()?.wait()?;
            }
            _ => {
                log::info!("No command provided, printing out matched target.");
                let target = scored_targets.last().ok_or("No targets")?;
                println!("{}", target);
            }
        };
        Ok(())
    }
}
