use strum_macros::EnumIter;

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
    fn new(build: T, host: T) -> Self {
        Self { build, host }
    }

    fn getitem(&self, machine: MachineChoice) -> &T {
        match machine {
            MachineChoice::Build => &self.build,
            MachineChoice::Host => &self.host,
        }
    }

    fn setitem(&mut self, machine: MachineChoice, val: T) {
        match machine {
            MachineChoice::Build => self.build = val,
            MachineChoice::Host => self.host = val,
        }
    }
}
