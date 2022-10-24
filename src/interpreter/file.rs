use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub struct File {
    pub filename: String,
    subdir: String,
}

impl File {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            ..Default::default()
        }
    }

    pub fn rel_to_builddir(&self, build_to_src: &Path) -> PathBuf {
        build_to_src.join(&self.subdir).join(&self.filename)
    }
}
