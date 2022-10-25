mod cpp;

use std::process::Command;

use assert_cmd::prelude::OutputAssertExt;

use crate::{environment::Environment, utils::MachineChoice};

use self::cpp::CPPCompiler;

#[derive(Clone, Default)]
pub enum CompilerType {
    #[default]
    CCompiler,
    CPPCompiler(CPPCompiler),
}

#[derive(Clone, Default)]
pub struct Compiler {
    pub id: String,

    exelist: Vec<String>,
    version: String,

    compiler_type: CompilerType,
}

impl Compiler {
    pub fn new(exelist: Vec<String>, version: &str, compiler_type: &CompilerType) -> Self {
        Self {
            exelist,
            version: version.to_owned(),
            compiler_type: compiler_type.to_owned(),
            ..Default::default()
        }
    }

    pub fn get_exelist(&self) -> &Vec<String> {
        &self.exelist
    }

    pub fn detect_compiler_for(
        env: &mut Environment,
        lang: &str,
        for_machine: &MachineChoice,
    ) -> Option<Compiler> {
        let comp = Self::compiler_from_language(env, lang, for_machine);
        if let Some(compiler) = &comp {
            let envc = env.to_owned();
            env.coredata.process_new_compiler(lang, compiler, &envc);
        }

        comp
    }

    fn compiler_from_language(
        env: &Environment,
        lang: &str,
        for_machine: &MachineChoice,
    ) -> Option<Compiler> {
        match lang {
            "c" => Self::detect_c_or_cpp_compiler(env, lang, for_machine),
            "cpp" => Self::detect_c_or_cpp_compiler(env, lang, for_machine),
            _ => todo!(),
        }
    }

    fn detect_c_or_cpp_compiler(
        env: &Environment,
        lang: &str,
        machine: &MachineChoice,
    ) -> Option<Compiler> {
        let compilers = Self::get_compilers(env, lang, machine);

        for compiler in compilers {
            // let compiler_name = PathBuf::from(&compiler).file_name();

            let mut cmd = Command::new(&compiler);
            cmd.arg("--version");
            cmd.assert().success();
            let out = cmd
                .output()
                .expect("Failed to get detected compiler output");
            let stdout = String::from_utf8(out.stdout).expect("Failed to convert stdout to string");

            let mut guess_gcc = None;
            if stdout.contains("Free Software Foundation") {
                guess_gcc = Some("gcc");
            }

            if guess_gcc.is_some() {
                if lang == "cpp" {
                    let compiler_t = CPPCompiler {};

                    let compiler = Compiler::new(
                        vec![compiler],
                        "1.11.1",
                        &CompilerType::CPPCompiler(compiler_t),
                    );
                    return Some(compiler);
                } else {
                    todo!("C Compiler")
                }
            }

            if stdout.contains("clang") {
                let compiler_t = CPPCompiler {};

                let compiler = Compiler::new(
                    vec![compiler],
                    "version",
                    &CompilerType::CPPCompiler(compiler_t),
                );

                return Some(compiler);
            }
        }

        None
    }

    fn get_compilers(env: &Environment, lang: &str, machine: &MachineChoice) -> Vec<String> {
        let value = env.lookup_binary_entry(machine, lang);
        match value {
            Some(compiler_var) => vec![compiler_var.to_owned()],
            None => panic!("No compiler found"),
        }
    }
}
