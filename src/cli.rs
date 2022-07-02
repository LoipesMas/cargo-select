use std::{error::Error, path::Path};

use clap::{Args, Parser, Subcommand};
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::get_tests::get_tests_from_path;
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
    #[clap(
        value_parser,
        long = "no-skip",
        help = "Run all tests that match selected test (i.e. dont skip names that are supersets)(tests only)"
    )]
    pub no_skip: bool,
}
impl Cli {
    pub fn exec(mut self) -> Result<(), Box<dyn Error>> {
        let Commands::SelectCommand(ref mut command) = self.command;
        let manifest = new_complete_manifest_from_path(Path::new("."))?;
        let targets = if matches!(command.cargo_command.as_deref(), Some("t") | Some("test")) {
            get_tests_from_path(Path::new("."))
        } else {
            targets_from_manifest(&manifest, Path::new("."))
        };
        let selected_target = match command.pattern.take() {
            Some(pattern) => score_targets(&targets, &pattern, &SkimMatcherV2::default())
                .last()
                .ok_or("No targets matched!")?,
            None => Tui::launch(&targets)?,
        };
        self.do_stuff_with_targets(&targets, selected_target)
    }

    fn do_stuff_with_targets(
        &self,
        targets: &[Target],
        selected_target: &Target,
    ) -> Result<(), Box<dyn Error>> {
        let Commands::SelectCommand(command) = &self.command;
        match command.cargo_command.as_deref() {
            Some("run") | Some("r") => {
                log::info!("Selected target: {selected_target}.");
                println!("Selected target: {selected_target}");
                log::debug!("Creating cargo command.");
                let mut proc_command = std::process::Command::new("cargo");
                let (name, workspace_path) = match selected_target {
                    Target::Bin(t) => (&t.name, &t.workspace_path),
                    Target::Example(t) => (&t.name, &t.workspace_path),
                    Target::Test(_) => unreachable!("You can't get test with `run` command."),
                };
                proc_command
                    .current_dir(&workspace_path)
                    .arg("run")
                    .arg(selected_target.to_cargo_flag())
                    .arg(name)
                    .args(&command.cargo_args);

                log::info!(
                    "Spawning cargo command: {proc_command:?} in {:#?}",
                    workspace_path
                );
                proc_command.spawn()?.wait()?;
            }
            Some("t") | Some("test") => {
                let (name, workspace_path) = match selected_target {
                    Target::Test(t) => (&t.name, &t.path),
                    _ => unreachable!("You can only get tests with `test` command."),
                };
                let to_skip = targets
                    .iter()
                    .filter_map(|t| {
                        if let Target::Test(t) = t {
                            if &t.name != name && t.name.contains(name) {
                                Some(["--skip", &t.name])
                            } else {
                                None
                            }
                        } else {
                            unreachable!("You can only get tests with `test` command.")
                        }
                    })
                    .flatten();
                log::info!("Selected target: {selected_target}.");
                println!("Selected target: {selected_target}");
                log::debug!("Creating cargo command.");
                let mut proc_command = std::process::Command::new("cargo");
                proc_command
                    .current_dir(&workspace_path.parent().unwrap())
                    .arg("test")
                    .arg(name)
                    .args(&command.cargo_args);

                if !command.no_skip {
                    proc_command.arg("--").args(to_skip);
                }

                log::info!(
                    "Spawning cargo command: {proc_command:?} in {:#?}",
                    workspace_path
                );
                proc_command.spawn()?.wait()?;
            }
            _ => {
                log::info!("No command provided, printing out matched target.");
                println!("{}", selected_target);
            }
        };
        Ok(())
    }
}
