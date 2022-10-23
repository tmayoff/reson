use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct File {
    subdir: String,
    filename: String,
}

impl File {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            ..Default::default()
        }
    }

    pub fn rel_to_builddir(&self, build_to_src: &PathBuf) -> PathBuf {
        build_to_src.join(&self.subdir).join(&self.filename)
    }
}
