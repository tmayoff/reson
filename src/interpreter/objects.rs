use lazy_static::lazy_static;

use crate::build::Target;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Elementary(ElementaryTypes),
    BuiltinTypes,
    ReturnedTypes(ReturnedObjectTypes),
}

impl Object {
    pub fn method_call(&self, method_name: &str, args: Vec<ElementaryTypes>) -> Object {
        match self {
            Object::Elementary(e) => Object::Elementary(e.method_call(method_name, args)),
            Object::BuiltinTypes => todo!(),
            Object::ReturnedTypes(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BuiltinTypes {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnedObjectTypes {
    Target(Target),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementaryTypes {
    Void,
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

                    panic!("Incorrect arguments for str.endswith()");
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

                    panic!("Incorrect arguments for str.join()");
                }
                "replace" => {
                    assert_eq!(args.len(), 2);

                    let mut it = args.iter();
                    let pattern = it.next().unwrap().str().unwrap();
                    let replace = it.next().unwrap().str().unwrap();

                    ElementaryTypes::Str(str.replace(&pattern, &replace))
                }
                "split" => {
                    assert_eq!(args.len(), 1);
                    let pat = args.first().unwrap();
                    if let ElementaryTypes::Str(pat) = pat {
                        let strs = str
                            .split(pat)
                            .map(|s| ElementaryTypes::Str(s.to_string()))
                            .collect();
                        return ElementaryTypes::List(strs);
                    }
                    panic!("Incorrect arguments for str.split()");
                }
                "startswith" => {
                    assert_eq!(args.len(), 1);

                    let pattern = args.first().unwrap().str().unwrap();
                    return ElementaryTypes::Bool(str.starts_with(&pattern));
                }

                "strip" => {
                    assert!(args.len() < 2);

                    if args.len() == 1 {
                        let chars = args.first().unwrap();
                        if let ElementaryTypes::List(chars) = chars {
                            let chars = chars
                                .iter()
                                .map(|c| c.str().unwrap())
                                .collect::<Vec<_>>()
                                .join("");

                            let s = regex::Regex::new(&format!("[{}]", chars)).unwrap();
                            let stripped = s.replace_all(str, "").to_string();
                            return ElementaryTypes::Str(stripped);
                        }
                    } else {
                        return ElementaryTypes::Str(str.split_whitespace().collect());
                    }

                    panic!("Incorrect arguments for str.strip()");
                }
                "substring" => {
                    assert!(args.len() < 3);

                    if args.len() == 1 {
                        let mut end: i32 = args.first().unwrap().int().unwrap();
                        if end < 0 {
                            end = str.len() as i32 - end;
                        }

                        return ElementaryTypes::Str(str.as_str()[..end as usize].to_string());
                    } else if args.len() == 2 {
                        let mut it = args.iter();
                        let mut start = it.next().unwrap().int().unwrap();
                        if start < 0 {
                            start = str.len() as i32 + start;
                        }

                        let mut end: i32 = it.next().unwrap().int().unwrap();
                        if end < 0 {
                            end = str.len() as i32 + end;
                        }

                        return ElementaryTypes::Str(
                            str.as_str()[start as usize..end as usize].to_string(),
                        );
                    }

                    panic!("Incorrect arguments for str.substring()");
                }
                "to_int" => {
                    return ElementaryTypes::Int(str.parse().expect("Failed to parse string"));
                }
                "to_lower" => {
                    return ElementaryTypes::Str(str.to_lowercase());
                }
                "to_upper" => {
                    return ElementaryTypes::Str(str.to_uppercase());
                }
                "underscorify" => {
                    lazy_static! {
                        static ref RE: regex::Regex = regex::Regex::new("[^a-zA-Z0-9]").unwrap();
                    }

                    return ElementaryTypes::Str(RE.replace_all(str, "_").to_string());
                }
                _ => panic!("Method {method_name}, unknown on type string"),
            },
            _ => panic!("Type has no methods"),
        }
    }

    fn int(&self) -> Option<i32> {
        match self {
            ElementaryTypes::Int(v) => Some(v.to_owned()),
            _ => None,
        }
    }

    fn str(&self) -> Option<String> {
        match self {
            ElementaryTypes::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

pub fn unholder(object: &Object) -> ElementaryTypes {
    match object {
        Object::Elementary(e) => e.to_owned(),
        Object::BuiltinTypes => todo!(),
        Object::ReturnedTypes(_) => todo!(),
    }
}
