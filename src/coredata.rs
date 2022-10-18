use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::utils::MachineChoice;

type CompilersType = HashMap<MachineChoice, HashMap<String, Compiler>>;

#[derive(Clone, Default)]
pub struct CoreData {
    lang_guids: HashMap<String, String>,

    compilers: CompilersType,
}

impl CoreData {
    pub fn new() -> Self {
        let lang_guids = HashMap::from([
            (
                "default".to_string(),
                "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942".to_string(),
            ),
            (
                "c".to_string(),
                "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942".to_string(),
            ),
            (
                "cpp".to_string(),
                "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942".to_string(),
            ),
            (
                "test".to_string(),
                "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942".to_string(),
            ),
            (
                "directory".to_string(),
                "2150E333-8FDC-42A3-9474-1A3956D46DE8".to_string(),
            ),
        ]);

        Self {
            lang_guids,
            ..Default::default()
        }
    }

    pub fn get_compilers(&self) -> &CompilersType {
        &self.compilers
    }
}
