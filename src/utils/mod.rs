use strum_macros::EnumIter;

use crate::{compiler::Compiler, interpreter::file::File};

#[derive(Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum MachineChoice {
    Build,
    Host,
}

#[derive(Clone, Default)]
pub struct PerMachine<T> {
    build: T,
    host: T,
}

impl<T> PerMachine<T> {
    pub fn new(build: T, host: T) -> Self {
        Self { build, host }
    }

    pub fn getitem(&self, machine: MachineChoice) -> &T {
        match machine {
            MachineChoice::Build => &self.build,
            MachineChoice::Host => &self.host,
        }
    }

    pub fn setitem(&mut self, machine: MachineChoice, val: T) {
        match machine {
            MachineChoice::Build => self.build = val,
            MachineChoice::Host => self.host = val,
        }
    }
}

pub fn get_compiler_for(compilers: &[Compiler], src: &File) -> Compiler {
    for comp in compilers {
        if comp.can_compile(src) {
            return comp.to_owned();
        }
    }

    panic!("No specified compiler can handle file {}", src);
}
