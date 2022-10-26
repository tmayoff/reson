use super::file::File;

#[derive(Clone)]
pub enum ObjectTypes {
    Builtin(BuiltinTypes),
    Returned(ReturnedObjectTypes),
    Elementary(ElementaryTypes),
}

#[derive(Clone)]
pub enum BuiltinTypes {}

#[derive(Clone)]
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
        ObjectTypes::Builtin(_) => todo!(),
        ObjectTypes::Returned(_) => todo!(),
        ObjectTypes::Elementary(e) => e.to_owned(),
    }
}
