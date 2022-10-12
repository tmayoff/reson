use std::fs;

use crate::{build::Build, BUILD_FILE_NAME};

struct InterpreterBase {}

pub struct Interpreter {
    build: Build,

    subdir: String,
    subproject: String,
    subproject_dir: String,

    code: String,
}

impl Interpreter {
    pub fn new(
        build: Build,
        backend: Option<String>,
        subdir: Option<String>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error> {
        let mut s = Self {
            build,
            subdir: subdir.unwrap_or_default(),
            subproject: subproject.unwrap_or_default(),
            subproject_dir: subproject_dir.unwrap_or_else(|| String::from("subprojects")),
            code: String::new(),
        };

        s.load_root_meson_file()?;

        Ok(s)
    }

    fn load_root_meson_file(&mut self) -> Result<(), std::io::Error> {
        let mut mesonfile = self.build.environment.source_dir.clone();
        mesonfile.push(self.subdir.clone());
        mesonfile.push(BUILD_FILE_NAME);

        self.code = fs::read_to_string(mesonfile)?;

        Ok(())
    }
}
