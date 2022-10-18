use std::path::PathBuf;

use crate::coredata::CoreData;

const PRIVATE_DIR: &str = "meson-private";
const LOG_DIR: &str = "meson-logs";
const INFO_DIR: &str = "meson-info";

#[derive(Clone, Default)]
pub struct Environment {
    pub source_dir: PathBuf,
    pub build_dir: Option<PathBuf>,

    coredata: CoreData,

    scratch_dir: PathBuf,
    info_dir: PathBuf,
    log_dir: PathBuf,
}

impl Environment {
    pub fn new(source_dir: PathBuf, build_dir: Option<&PathBuf>) -> Result<Self, std::io::Error> {
        let mut env = Self {
            source_dir,
            ..Default::default()
        };

        if let Some(build_dir) = build_dir {
            env.build_dir = Some(build_dir.clone());

            env.scratch_dir = build_dir.clone();
            env.scratch_dir.push(PRIVATE_DIR);
            env.info_dir = build_dir.clone();
            env.info_dir.push(INFO_DIR);
            env.log_dir = build_dir.clone();
            env.log_dir.push(LOG_DIR);

            std::fs::create_dir_all(&env.scratch_dir)?;
            std::fs::create_dir_all(&env.info_dir)?;
            std::fs::create_dir_all(&env.log_dir)?;

            // env.core_data = coredata.load(env.get_build_dir());
        }

        Ok(env)
    }

    pub fn get_build_command() -> Vec<String> {
        let cmd = std::env::current_exe().expect("Couldn't find the current exe");
        vec![cmd.to_string_lossy().to_string()]
    }

    pub fn get_ninja_command_and_version(_version: Option<&str>, _log: Option<bool>) -> PathBuf {
        // let version = version.unwrap_or("1.8.2");
        // let log = log.unwrap_or(false);

        match which::which("ninja") {
            Ok(p) => p,
            Err(_) => panic!("Binary can't be found"),
        }
    }

    pub fn get_build_dir(&self) -> &Option<PathBuf> {
        &self.build_dir
    }

    pub fn get_coredata(&self) -> &CoreData {
        &self.coredata
    }

    fn create_new_coredata() {}
}
