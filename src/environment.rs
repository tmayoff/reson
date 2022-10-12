use std::path::PathBuf;

#[derive(Clone)]
pub struct Environment {
    source_dir: Option<PathBuf>,
    build_dir: Option<PathBuf>,
}

impl Environment {
    pub fn new(source_dir: Option<PathBuf>, build_dir: Option<PathBuf>) -> Self {
        Self {
            source_dir,
            build_dir,
        }
    }

    fn create_new_coredata() {}
}
