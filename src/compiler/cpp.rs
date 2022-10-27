use super::CompilerTypeTrait;

#[derive(Clone, Debug)]
pub struct CPPCompiler {}

impl CompilerTypeTrait for CPPCompiler {
    fn get_depfile_suffix(&self) -> String {
        "d".to_string()
    }
}
