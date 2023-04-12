#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Elementary(ElementaryTypes),
}

impl Object {
    pub fn method_call(&self, method_name: &str) -> Object {
        match self {
            Object::Elementary(e) => Object::Elementary(e.method_call(method_name)),
        }
    }
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

impl ElementaryTypes {
    fn method_call(&self, method_name: &str) -> ElementaryTypes {
        match self {
            ElementaryTypes::Bool(b) => match method_name {
                "to_int" => ElementaryTypes::Int((*b).into()),
                "to_string" => todo!(),
                _ => panic!("Method unknown on type bool"),
            },
            ElementaryTypes::Dict => todo!(),
            ElementaryTypes::Int(_) => todo!(),
            ElementaryTypes::List(_) => todo!(),
            ElementaryTypes::Str(_) => todo!(),
        }
    }
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
