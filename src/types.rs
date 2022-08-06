#[derive(Debug, Clone, PartialEq)]
pub enum Sort {
    Default,
    Ascending,
    Descending,
}

impl From<&str> for Sort {
    fn from(s: &str) -> Self {
        match s {
            ">" => Sort::Descending,
            "<" => Sort::Ascending,
            _ => Sort::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparsedObject {
    pub fields: Vec<String>,
    pub parent: Option<Box<SparsedObject>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub name: String,
    pub fields: Vec<Field>,
    pub parent: Option<Box<Object>>,
    pub sort: Sort,
    pub has_sparsed_fields: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Str,
    Int,
    Float,
    Bool,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Epoch,
    Path,
    Reducer {
        which: Box<Type>,
        start: i32,
        end: i32,
    },
    ThisRef {
        object_field_name: String,
        append_type: Option<Box<Type>>,
    },
    Ref {
        object_name: String,
        field_name: String,
    },
    This {
        sparsed_object: SparsedObject,
        // parent: Option<SparsedObject>,
    },
    Object {
        object_name: String,
    },
    Null,
    NullType {
        which: Box<Type>,
    },
}

impl Type {
    pub fn is_primitive(str: &str) -> bool {
        match str {
            "str" | "int" | "float" | "bool" | "{@epoch}" | "{@path}" => true,
            _ => false,
        }
    }

    pub fn can_be_reduced_str(string: &str) -> bool {
        if string.starts_with("str")
            || string.starts_with("int")
            || string.starts_with("float")
            || string.starts_with("{@epoch}")
            || string.starts_with("{@path}")
        {
            true
        } else if string.starts_with("[") {
            let is_reducer = string.contains("..");
            if is_reducer {
                return false;
            }
            //TODO: check if the type is valid, Add additional checks
            true
        } else {
            false
        }
    }

    pub fn has_reducer(string: &str) -> bool {
        if Type::can_be_reduced_str(string) {
            if string.contains("..") {
                return true;
            }
            false
        } else {
            false
        }
    }

    pub fn get_reduced_type_from_str(string: &str) -> Type {
        let (start, end) = Type::get_reducer(string);
        let which = {
            let mut splits = string.split("[");
            let which = splits.next().unwrap();
            Box::new(Type::from_str(which))
        };
        Type::Reducer { which, start, end }
    }

    pub fn get_reducer(string: &str) -> (i32, i32) {
        // "str[0"     "2]"
        // "str["      "2]"
        let mut splitted = string.split("..");

        // prime = str[0
        // prime = str[
        let prime: String = splitted.next().unwrap().chars().rev().collect();

        // prime = 0[rts
        let mut start_rev = prime.split("[");
        // println!("{:?}", start_rev);
        let start = {
            if let Some(start) = start_rev.next() {
                if start.is_empty() {
                    0
                } else {
                    start
                        .chars()
                        .rev()
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap()
                }
            } else {
                0
            }
        };

        // end = 2]
        let end = splitted
            .next()
            .unwrap()
            .split("]")
            .next()
            .unwrap()
            .parse::<i32>()
            .unwrap();

        (start, end)
    }

    pub fn is_map(string: &str) -> bool {
        if string.contains(":") {
            true
        } else {
            false
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        let can_be_reduced = Type::can_be_reduced_str(s);
        let has_reducer = Type::has_reducer(s);
        let has_field_ref = s.contains("}.");
        let has_self_field_ref = s.contains("{this.");
        let mut has_sparsed_obj_ref = false;
        if !has_self_field_ref {
            if s.contains("this.") {
                has_sparsed_obj_ref = true;
            }
        }
        if (!can_be_reduced || !has_reducer)
            && !has_field_ref
            && !has_self_field_ref
            && !has_sparsed_obj_ref
        {
            match s {
                "int" => Type::Int,
                "float" => Type::Float,
                "str" => Type::Str,
                "bool" => Type::Bool,
                "{@epoch}" => Type::Epoch,
                "{@path}" => Type::Path,
                // "{@ref}" => Type::Ref(Box::new(Type::Str)),
                // "{this}" => Type::ThisRef,
                // "this" => Type::This,
                "!" => Type::Null,
                s => {
                    if s.starts_with('!') {
                        let object_name = s[1..].to_string();
                        let is_primitive = Type::is_primitive(&object_name);
                        let which = if is_primitive {
                            Type::from_str(&object_name)
                        } else {
                            Type::Object { object_name }
                        };
                        Type::NullType {
                            which: Box::new(which),
                        }
                    } else if Type::is_map(s) {
                        //[@key      value]
                        let mut splitted = s.split(":");
                        let key = splitted.next().unwrap().to_string();
                        let value = splitted.next().unwrap().split(']').next().unwrap();
                        // let mut value = splitted.next().unwrap().to_string();
                        // value.reserve_exact(0);
                        // let mut rev = String::from(&value[1..]);
                        // rev.reserve_exact(0);
                        Type::Map(
                            Box::new(Type::from_str(&key[2..])),
                            Box::new(Type::from_str(&value)),
                        )
                    } else {
                        let s = s.split("]").next().unwrap();
                        Type::List(Box::new(Type::from_str(&s[1..])))
                    }
                }
            }
        } else if has_self_field_ref {
            let mut splits = s.split("{this.").skip(1);
            // let object_name = (&splits.next().unwrap()[5..]).to_string();
            // let which = Type::from_str(object_name);;
            let mut splits = splits.next().unwrap().split(".");
            let object_field_name = splits.next().unwrap().to_string();
            let r = &splits.next().unwrap().chars().rev().collect::<String>()[1..];
            let append_type = {
                let n = &r.chars().rev().collect::<String>()[..];
                if n.is_empty() {
                    None
                } else {
                    Some(Box::new(Type::from_str(n)))
                }
            };

            Type::ThisRef {
                object_field_name,
                append_type,
            }
        } else if has_sparsed_obj_ref {
            let mut splits = s.split("this.").skip(1);
            // let object_name = (&splits.next().unwrap()[5..]).to_string();
            // let which = Type::from_str(object_name);;
            // let mut splits = splits.next().unwrap().split(".");
            let object_field_name = splits.next().unwrap().to_string();
            let r = &object_field_name;
            let r = &r[1..r.len() - 1];
            let fields = r.split(",").collect::<Vec<&str>>();
            let has_parent = fields[0].contains("..");
            let parent: Option<Box<SparsedObject>> = if has_parent {
                // let parent = fields[0].split("..").skip(1).next().unwrap();
                // fields.push(parent);
                None
            } else {
                None
            };
            let sparsed_object = SparsedObject {
                parent,
                fields: fields.iter().map(|x| x.to_string()).collect(),
            };

            Type::This { sparsed_object }
        } else if has_field_ref {
            let mut splits = s.split("}.");
            let object_name = (&splits.next().unwrap()[6..]).to_string();
            // let which = Type::from_str(object_name);
            let field_name = splits.next().unwrap().to_string();

            Type::Ref {
                object_name,
                field_name,
            }
        } else {
            Type::get_reduced_type_from_str(s)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_type_from_str() {
        let int_type = super::Type::from_str("int");
        let float_type = super::Type::from_str("float");
        let str_type = super::Type::from_str("str");
        let bool_type = super::Type::from_str("bool");
        let epoch_type = super::Type::from_str("{@epoch}");
        let path_type = super::Type::from_str("{@path}");
        // let ref_type = super::Type::from_str("{@ref}");
        // let thisref_type = super::Type::from_str("{this}");
        // let this_type = super::Type::from_str("this");
        let null_type = super::Type::from_str("!");
        let list_type = super::Type::from_str("[str]");
        let map_type = super::Type::from_str("[@str:str]");

        assert_eq!(int_type, super::Type::Int);
        assert_eq!(float_type, super::Type::Float);
        assert_eq!(str_type, super::Type::Str);
        assert_eq!(bool_type, super::Type::Bool);
        assert_eq!(epoch_type, super::Type::Epoch);
        assert_eq!(path_type, super::Type::Path);
        // assert_eq!(ref_type, super::Type::Ref(Box::new(super::Type::Str)));
        // assert_eq!(thisref_type, super::Type::ThisRef);
        // assert_eq!(this_type, super::Type::This);
        assert_eq!(null_type, super::Type::Null);
        assert_eq!(list_type, super::Type::List(Box::new(super::Type::Str)));
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Str))
        );
    }

    #[test]
    fn test_map_type_combinations_from_str() {
        let map_type = super::Type::from_str("[@str:str]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Str))
        );

        let map_type = super::Type::from_str("[@str:int]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Int))
        );

        let map_type = super::Type::from_str("[@str:float]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Float))
        );

        let map_type = super::Type::from_str("[@str:bool]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Bool))
        );

        let map_type = super::Type::from_str("[@str:{@epoch}]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Epoch))
        );

        let map_type = super::Type::from_str("[@str:{@path}]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Path))
        );

        // let map_type = super::Type::from_str("[@str:{@ref}]");
        // assert_eq!(
        //     map_type,
        //     super::Type::Map(
        //         Box::new(super::Type::Str),
        //         Box::new(super::Type::Ref(Box::new(super::Type::Str)))
        //     )
        // );

        // let map_type = super::Type::from_str("[@str:{this}]");
        // assert_eq!(
        //     map_type,
        //     super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::ThisRef))
        // );

        // let map_type = super::Type::from_str("[@str:this]");
        // assert_eq!(
        //     map_type,
        //     super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::This))
        // );

        let map_type = super::Type::from_str("[@str:!]");
        assert_eq!(
            map_type,
            super::Type::Map(Box::new(super::Type::Str), Box::new(super::Type::Null))
        );

        let map_type = super::Type::from_str("[@str:[str]]");
        assert_eq!(
            map_type,
            super::Type::Map(
                Box::new(super::Type::Str),
                Box::new(super::Type::List(Box::new(super::Type::Str)))
            )
        );
    }
}
