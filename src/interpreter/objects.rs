#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Elementary(ElementaryTypes),
    BuiltinTypes,
}

impl Object {
    pub fn method_call(&self, method_name: &str, args: Vec<ElementaryTypes>) -> Object {
        match self {
            Object::Elementary(e) => Object::Elementary(e.method_call(method_name, args)),
            Object::BuiltinTypes => todo!(),
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
    fn method_call(&self, method_name: &str, args: Vec<ElementaryTypes>) -> ElementaryTypes {
        match self {
            ElementaryTypes::Bool(b) => match method_name {
                "to_int" => ElementaryTypes::Int((*b).into()),
                "to_string" => ElementaryTypes::Str(b.to_string()),
                _ => panic!("Method {method_name} unknown on type bool"),
            },
            ElementaryTypes::Dict => todo!(),
            ElementaryTypes::Int(i) => match method_name {
                "is_even" => ElementaryTypes::Bool(i % 2 == 0),
                "is_odd" => ElementaryTypes::Bool(i % 2 != 0),
                "to_string" => ElementaryTypes::Str(i.to_string()),
                _ => panic!("Method {method_name} unknown on type int"),
            },
            ElementaryTypes::List(_) => todo!(),
            ElementaryTypes::Str(str) => match method_name {
                "contains" => {
                    assert_eq!(args.len(), 1);

                    if let ElementaryTypes::Str(pattern) = args.first().unwrap() {
                        return ElementaryTypes::Bool(str.contains(pattern));
                    }

                    panic!("Incorrect arguments for str.contains()");
                }
                "endswith" => {
                    assert_eq!(args.len(), 1);

                    if let ElementaryTypes::Str(pattern) = args.first().unwrap() {
                        return ElementaryTypes::Bool(str.ends_with(pattern));
                    }

                    panic!("Incorrect arguments for str.contains()");
                }
                "join" => {
                    assert_eq!(args.len(), 1);

                    let mut strings = Vec::new();
                    if let ElementaryTypes::List(objs) = args.first().unwrap() {
                        for o in objs {
                            if let ElementaryTypes::Str(c) = o {
                                strings.push(c.to_owned());
                            }
                        }

                        return ElementaryTypes::Str(strings.join(str));
                    }

                    panic!("Incorrect arguments for str.contains()");
                }
                "replace" => {
                    assert_eq!(args.len(), 2);

                    let mut it = args.iter();
                    let pattern = it.next().unwrap().str().unwrap();
                    let replace = it.next().unwrap().str().unwrap();

                    ElementaryTypes::Str(str.replace(&pattern, &replace))
                }
                _ => panic!("Method {method_name}, unknown on type string"),
            },
        }
    }

    fn str(&self) -> Option<String> {
        match self {
            ElementaryTypes::Str(s) => Some(s.to_string()),
            _ => None,
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
        Object::BuiltinTypes => todo!(),
    }
}
