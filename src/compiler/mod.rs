use std::process::Command;

use crate::{environment::Environment, utils::MachineChoice};

#[derive(Clone, Default)]
pub struct Compiler {
    exelist: Vec<String>,
    version: String,
    id: String,
}

impl Compiler {
    pub fn new(exelist: Vec<String>, version: &str) -> Self {
        Self {
            exelist,
            version: version.to_owned(),
            ..Default::default()
        }
    }

    pub fn get_exelist(&self) -> &Vec<String> {
        &self.exelist
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn detect_compiler_for(env: &Environment, lang: String, for_machine: MachineChoice) {
        let comp = Self::compiler_from_language(env, lang.as_str(), for_machine);
    }

    fn compiler_from_language(
        env: &Environment,
        lang: &str,
        for_machine: MachineChoice,
    ) -> Option<Compiler> {
        match lang {
            "c" => todo!(),
            "cpp" => todo!(),
            _ => None,
        }
    }

    fn detect_c_or_cpp_compiler(env: &Environment, lang: &str, machine: MachineChoice) {
        let compiler = Self::get_compilers(env, lang, machine);

        let cmd = Command::new(compiler + " --version");
        // let out = cmd.spawn();
    }

    fn get_compilers(env: &Environment, lang: &str, machine: MachineChoice) -> String {
        let value = env.lookup_binary_entry(machine, lang);
        match value {
            Some(compiler_var) => compiler_var.to_owned(),
            None => panic!("No compiler found"),
        }
    }
}
