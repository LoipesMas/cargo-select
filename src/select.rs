use cargo_toml::{Manifest, Product};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use std::{cmp::Reverse, error::Error, path::Path};

#[derive(Debug)]
pub enum TargetType {
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
pub struct Target {
    pub name: String,
    pub path: String,
    pub target_type: TargetType,
}

impl Target {
    pub fn new(product: &Product, path: &Path, target_type: TargetType) -> Self {
        Self {
            name: product.name.to_owned().unwrap_or_default(),
            path: path
                .join(product.path.to_owned().unwrap_or_default())
                .to_string_lossy()
                .to_string(),
            target_type,
        }
    }
    pub fn fuzzy_match(&self, pattern: &str, skim: &SkimMatcherV2) -> i64 {
        skim.fuzzy_match(&self.to_string(), pattern).unwrap_or(-1)
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:30}\t({})", self.target_type, self.name, self.path)
    }
}

pub fn targets_from_manifest(manifest: &Manifest, path: &Path) -> Vec<Target> {
    log::debug!("Getting targets from manifest.");
    let mut ret = vec![];
    for bin in &manifest.bin {
        let target = Target::new(bin, path, TargetType::Bin);
        log::debug!("Adding target: {}", target);
        ret.push(target);
    }
    for example in manifest.example.iter() {
        let target = Target::new(example, path, TargetType::Example);
        log::debug!("Adding target: {}", target);
        ret.push(target);
    }
    if let Some(workspace) = &manifest.workspace {
        for member in &workspace.members {
            log::debug!("Handling workspace: {member}.");
            if let Some(member) = member.strip_suffix("/*") {
                // new_complete_manifest_from_path for all subdirs
                log::warn!("Unhandled member directory: {member:#?}!");
            } else {
                let member_path = &path.join(member);
                let manifest = new_complete_manifest_from_path(member_path).unwrap();
                ret.append(&mut targets_from_manifest(&manifest, member_path));
            }
        }
    }
    ret
}

pub fn score_targets<'a>(
    targets: &'a [Target],
    pattern: &str,
    skim: &SkimMatcherV2,
) -> Vec<&'a Target> {
    log::debug!("Scoring targets with pattern: {pattern}.");
    let mut ret = targets
        .iter()
        .map(|target| (target, target.fuzzy_match(pattern, skim)))
        .filter(|&(_target, score)| score > 0)
        .collect::<Vec<_>>();
    ret.sort_unstable_by_key(|&(target, _score)| Reverse(&target.name));
    ret.sort_by_key(|&(_, score)| score);
    ret.iter().map(|&(t, _)| t).collect()
}

pub fn new_complete_manifest_from_path(path: &Path) -> Result<Manifest, Box<dyn Error>> {
    log::info!("Getting complete manifest from path: {path:?}");
    let mut manifest = Manifest::from_path(path.join("Cargo.toml"))?;
    manifest.complete_from_path(Path::new("."))?;
    Ok(manifest)
}
