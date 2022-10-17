use std::collections::HashMap;

use crate::build::Build;

use super::InterpreterTrait;

#[derive(Clone)]
pub enum HoldableTypes {
    Dict,
    VecBool(Vec<bool>),
    VecInt(Vec<i32>),
    VecStr(Vec<String>),
    Boolean(bool),
    Str(String),
    Int(i32),
}

#[derive(Clone)]
pub struct ObjectHolder {
    pub held_object: HoldableTypes,
}

#[derive(Clone)]
pub struct MesonMain {
    // build: Build,
    // interpreter: Box<dyn InterpreterTrait>,
    // methods: HashMap<String, String>,
}

#[derive(Clone)]
pub enum Object {
    ObjectHolder(ObjectHolder),
    MesonMain(MesonMain),
}

#[derive(Clone)]
pub struct InterpreterObject {
    pub obj: Object,
}

impl InterpreterObject {
    pub fn new(obj: Object) -> Self {
        Self { obj }
    }

    pub fn meson_main(build: Build, interpreter: Box<dyn InterpreterTrait>) -> Self {
        Self {
            obj: Object::MesonMain(MesonMain {}),
        }
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
        Object::MesonMain(_) => todo!(),
    }
}
