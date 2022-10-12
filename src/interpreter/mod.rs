use crate::build::Build;

struct InterpreterBase {}

pub struct Interpreter {
    build: Build,

    subdir: String,
    subproject: String,
    subproject_dir: String,
}

impl Interpreter {
    pub fn new(
        build: Build,
        backend: Option<String>,
        subdir: Option<String>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Self {
        Self {
            build,
            subdir: subdir.unwrap_or_default(),
            subproject: subproject.unwrap_or_default(),
            subproject_dir: subproject_dir.unwrap_or(String::from("subprojects")),
        }
    }
}
