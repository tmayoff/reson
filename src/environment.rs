use std::path::PathBuf;

#[derive(Clone)]
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

    fn create_new_coredata() {}
}
