#[derive(Clone, Default)]
pub struct Compiler {
    id: String,
}

impl Compiler {
    pub fn get_id(&self) -> &String {
        &self.id
    }
}
