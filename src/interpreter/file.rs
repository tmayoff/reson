use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct File {
    pub filename: String,
    pub subdir: String,
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

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.filename)
    }
}
