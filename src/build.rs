use std::collections::HashMap;

use crate::environment::Environment;

#[derive(Default, Clone)]
pub enum Target {
    #[default]
    BuildTarget,
    CustomTarget,
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
