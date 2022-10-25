use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::utils::PerMachine;
use crate::{coredata::CoreData, utils::MachineChoice};
use lazy_static::lazy_static;
use strum::IntoEnumIterator;

const PRIVATE_DIR: &str = "meson-private";
const LOG_DIR: &str = "meson-logs";
const INFO_DIR: &str = "meson-info";

#[derive(Clone, Default)]
pub struct Environment {
    pub source_dir: PathBuf,
    pub build_dir: PathBuf,

    pub coredata: CoreData,

    scratch_dir: PathBuf,
    info_dir: PathBuf,
    log_dir: PathBuf,

    binaries: HashMap<String, String>,
}

impl Environment {
    pub fn new(source_dir: &Path, build_dir: &Path) -> Result<Self, std::io::Error> {
        let mut env = Self {
            source_dir: source_dir.to_owned(),
            ..Default::default()
        };

        env.build_dir = build_dir.to_owned();
        env.scratch_dir = build_dir.join(PRIVATE_DIR);
        env.info_dir = build_dir.join(INFO_DIR);
        env.log_dir = build_dir.join(LOG_DIR);

        std::fs::create_dir_all(&env.scratch_dir)?;
        std::fs::create_dir_all(&env.info_dir)?;
        std::fs::create_dir_all(&env.log_dir)?;

        // env.core_data = coredata.load(env.get_build_dir());

        env.set_default_binaries_from_env();

        Ok(env)
    }

    // pub fn get_build_command() -> Vec<String> {
    //     let cmd = std::env::current_exe().expect("Couldn't find the current exe");
    //     vec![cmd.to_string_lossy().to_string()]
    // }

    pub fn get_ninja_command_and_version(_version: Option<&str>, _log: Option<bool>) -> PathBuf {
        // let version = version.unwrap_or("1.8.2");
        // let log = log.unwrap_or(false);

        match which::which("ninja") {
            Ok(p) => p,
            Err(_) => panic!("Binary can't be found"),
        }
    }

    pub fn get_build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    fn set_default_binaries_from_env(&mut self) {
        for (name, evar) in ENV_VAR_COMPILER_MAP.iter() {
            let expanded = std::env::var(evar);
            if let Ok(expanded) = expanded {
                self.binaries.insert(name.to_string(), expanded);
            }
        }
    }

    //     fn create_new_coredata() {}

    pub fn lookup_binary_entry(&self, _machine: &MachineChoice, name: &str) -> Option<&String> {
        if self.binaries.contains_key(name) {
            return Some(&self.binaries[name]);
        }

        None
    }
}

lazy_static! {
    static ref ENV_VAR_COMPILER_MAP: HashMap<String, String> = HashMap::from([
        ("c".to_owned(), "CC".to_owned()),
        ("cpp".to_owned(), "CXX".to_owned()),
    ]);
}
