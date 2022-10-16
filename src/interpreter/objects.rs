pub enum HoldableTypes {
    Dict,
    VecBool(Vec<bool>),
    VecInt(Vec<i32>),
    VecStr(Vec<String>),
    Boolean(bool),
    Str(String),
    Int(i32),
}

pub struct ObjectHolder {
    pub held_object: HoldableTypes,
}

pub enum Object {
    ObjectHolder(ObjectHolder),
}

pub struct InterpreterObject {
    pub obj: Object,
}

impl InterpreterObject {
    pub fn new(obj: Object) -> Self {
        Self { obj }
    }

    pub fn object_holder(obj: HoldableTypes) -> Self {
        Self {
            obj: Object::ObjectHolder(ObjectHolder { held_object: obj }),
        }
    }
}

pub fn unholder(object: InterpreterObject) -> HoldableTypes {
    match object.obj {
        Object::ObjectHolder(obj_holder) => return obj_holder.held_object,
    }
}
