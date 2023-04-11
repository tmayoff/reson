use super::file::File;

#[derive(Clone)]
pub enum ObjectTypes {
    Elementary(ElementaryTypes),
}

#[derive(Clone)]
pub enum BuiltinTypes {}

#[derive(Clone, PartialEq, Eq)]
pub enum ElementaryTypes {
    Void,
    Bool(bool),
    Dict,
    Int(i32),
    List(Vec<ElementaryTypes>),
    Str(String),
}

#[derive(Clone)]
pub enum ReturnedObjectTypes {
    File(File),
}

pub fn unholder(object: &ObjectTypes) -> ElementaryTypes {
    match object {
        ObjectTypes::Elementary(e) => e.to_owned(),
    }
}
