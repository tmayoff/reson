use crate::environment::Environment;

#[derive(Default, Clone)]
pub struct Build {
    project_name: String,
    project_version: Option<String>,
    pub environment: Environment,
}

impl Build {
    pub fn new(env: Environment) -> Self {
        Self {
            project_name: String::from(""),
            project_version: None,
            environment: env,
        }
    }
}
