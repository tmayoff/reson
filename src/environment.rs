use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct Environment {
    pub source_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl Environment {
    pub fn new(source_dir: PathBuf, build_dir: PathBuf) -> Self {
        Self {
            source_dir,
            build_dir,
        }
    }

    pub fn get_coredata(&self) -> String {
        String::new()
    }

    fn create_new_coredata() {}
}
