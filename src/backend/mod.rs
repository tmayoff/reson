pub mod ninja;

pub trait Backend {
    fn generate(&mut self);

    fn get_name(&self) -> &String;
}
