#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Elementary(ElementaryTypes),
}

#[derive(Clone)]
pub enum BuiltinTypes {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementaryTypes {
    // Void,
    Bool(bool),
    Dict,
    Int(i32),
    List(Vec<ElementaryTypes>),
    Str(String),
}

#[derive(Clone)]
pub enum ReturnedObjectTypes {
    // File(File),
}

pub fn unholder(object: &Object) -> ElementaryTypes {
    match object {
        Object::Elementary(e) => e.to_owned(),
    }
}
