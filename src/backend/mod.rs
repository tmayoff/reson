pub trait Backend {
    fn get_name(&self) -> &String;

    fn generate(&self);
}

#[derive(Clone, Default)]
pub struct NinjaBackend {
    name: String,
}

impl NinjaBackend {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Backend for NinjaBackend {
    fn generate(&self) {
        todo!()
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
