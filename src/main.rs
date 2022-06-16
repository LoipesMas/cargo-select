use std::{cmp::Reverse, error::Error, path::Path};

use cargo_toml::Manifest;
use clap::{Args, Parser, Subcommand};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo")]
struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "select")]
    SelectCommand(SelectCommand),
}

#[derive(Args, Debug)]
struct SelectCommand {
    #[clap(value_parser)]
    pub name: Option<String>,
    #[clap(value_parser, requires = "name")]
    pub cargo_command: Option<String>,
    // TODO: make it a "last" argument, i.e. after "--"
    #[clap(value_parser)]
    pub cargo_args: Vec<String>,
}

#[derive(Debug)]
enum TargetType {
    Bin,
    Example,
}

impl TargetType {
    pub fn to_cargo_flag(&self) -> &'static str {
        match self {
            TargetType::Bin => "--package",
            TargetType::Example => "--example",
        }
    }
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetType::Bin => "Binary",
                TargetType::Example => "Example",
            }
        )
    }
}

#[derive(Debug)]
struct Target {
    pub name: String,
    pub path: String,
    pub target_type: TargetType,
}

impl Target {
    pub fn fuzzy_match(&self, pattern: &str, skim: &SkimMatcherV2) -> i64 {
        skim.fuzzy_match(&self.name, pattern).unwrap_or(-1)
            + skim.fuzzy_match(&self.path, pattern).unwrap_or(-1)
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:30}\t({})", self.target_type, self.name, self.path)
    }
}

fn targets_from_manifest(manifest: &Manifest, path: &Path) -> Vec<Target> {
    let mut ret = vec![];
    for bin in &manifest.bin {
        ret.push(Target {
            name: bin.name.to_owned().unwrap_or_default(),
            path: path
                .join(bin.path.to_owned().unwrap_or_default())
                .to_string_lossy()
                .to_string(),
            target_type: TargetType::Bin,
        });
    }
    for example in manifest.example.iter() {
        ret.push(Target {
            name: example.name.to_owned().unwrap_or_default(),
            path: path
                .join(example.path.to_owned().unwrap_or_default())
                .to_string_lossy()
                .to_string(),
            target_type: TargetType::Example,
        });
    }
    if let Some(workspace) = &manifest.workspace {
        for member in &workspace.members {
            if let Some(member) = member.strip_suffix("/*") {
                println!("WARN: Unhandled member: {member:#?}");
            } else {
                let member_path = &path.join(member);
                let manifest = new_complete_manifest_from_path(member_path).unwrap();
                ret.append(&mut targets_from_manifest(&manifest, member_path));
            }
        }
    }
    ret
}

fn new_complete_manifest_from_path(path: &Path) -> Result<Manifest, Box<dyn Error>> {
    let mut manifest = Manifest::from_path(path.join("Cargo.toml"))?;
    manifest.complete_from_path(Path::new("."))?;
    Ok(manifest)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let Commands::SelectCommand(command) = args.command;
    let manifest = new_complete_manifest_from_path(Path::new("."))?;
    let targets = targets_from_manifest(&manifest, Path::new("."));
    let skim = SkimMatcherV2::default();
    if command.name.is_none() {
        for target in &targets {
            println!("{}", target);
        }
        return Ok(());
    }
    let mut targets_zipped = targets
        .iter()
        .zip(
            targets
                .iter()
                .map(|t| t.fuzzy_match(command.name.as_ref().unwrap(), &skim)),
        )
        .collect::<Vec<_>>();
    targets_zipped.retain(|&(_, score)| score > 0);
    targets_zipped.sort_unstable_by_key(|&(target, _score)| Reverse(&target.name));
    targets_zipped.sort_by_key(|&(_, score)| score);
    match command.cargo_command.as_deref() {
        Some("run") => {
            let (target, _score) = targets_zipped.last().ok_or("No targets")?;
            println!("Selected target: {} ({})", target.name, target.target_type);
            let mut proc_command = std::process::Command::new("cargo");
            proc_command
                .arg("run")
                .arg(target.target_type.to_cargo_flag())
                .arg(&target.name)
                .args(command.cargo_args)
                .spawn()?
                .wait()?;
        }
        _ => {
            for (target, _score) in &targets_zipped {
                println!("{}", target);
            }
        }
    }
    Ok(())
}
