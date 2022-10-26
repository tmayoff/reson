use std::{collections::HashMap, path::PathBuf};

use crate::{environment::Environment, interpreter::file::File};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct BuildTarget {
    pub filename: String,
}

impl BuildTarget {
    pub fn new() -> Self {
        Self {
            filename: "no_name".to_owned(),
            // ..Default::default()
        }
    }

    fn get_dependencies(&self, target: &Target, exclude: Option<Vec<Target>>) -> Vec<Target> {
        let mut transitive_deps = Vec::new();

        let mut full_link_targets = target.link_targets.clone();
        full_link_targets.append(&mut target.link_whole_targets.clone());

        for t in full_link_targets {
            if transitive_deps.contains(&t) {
                continue;
            }
            transitive_deps.push(t.clone());

            if matches!(t.target_type, TargetType::StaticLibrary) {
                let mut e = exclude.clone().unwrap_or_default();
                let mut tr: Vec<Target> = transitive_deps.clone();
                tr.append(&mut e);
                let mut deps = t.get_dependencies(Some(tr)).to_owned();
                transitive_deps.append(&mut deps);
            }
        }

        transitive_deps
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TargetType {
    BuildTarget(BuildTarget),
    CustomTarget,
    SharedLibrary,
    StaticLibrary,
}

impl Default for TargetType {
    fn default() -> Self {
        Self::BuildTarget(BuildTarget::default())
    }
}

#[derive(Clone, Default)]
pub struct Target {
    pub target_type: TargetType,
    pub sources: Vec<File>,
    pub subdir: PathBuf,

    name: String,
    link_targets: Vec<Target>,
    link_whole_targets: Vec<Target>,
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Target {
    pub fn new(name: &str, target_type: &TargetType, subdir: &PathBuf, sources: &[File]) -> Self {
        Self {
            name: name.to_owned(),
            target_type: target_type.to_owned(),
            subdir: subdir.to_owned(),
            sources: sources.to_owned(),
            ..Default::default()
        }
    }

    pub fn get_id(&self) -> String {
        String::new()
    }

    pub fn get_dependencies(&self, exclude: Option<Vec<Target>>) -> Vec<Target> {
        match &self.target_type {
            TargetType::BuildTarget(b) => b.get_dependencies(self, exclude),
            TargetType::CustomTarget => todo!(),
            TargetType::SharedLibrary => todo!(),
            TargetType::StaticLibrary => todo!(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Build {
    pub project_name: String,
    project_version: Option<String>,
    pub environment: Environment,

    pub targets: HashMap<String, Target>,
}

impl Build {
    pub fn new(env: Environment) -> Self {
        Self {
            project_name: String::from(""),
            project_version: None,
            environment: env,
            ..Default::default()
        }
    }
}
