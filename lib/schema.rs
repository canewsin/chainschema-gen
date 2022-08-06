use std::path::Path;

use crate::types::{Field, Object, Sort, Type};

#[derive(Debug, Clone)]
pub enum SchemeError {
    PathNotExists,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub path: String,
    pub objects: Vec<Object>,
}

impl Schema {
    pub fn load(path: &Path) -> Result<Schema, SchemeError> {
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();

            // let mut object: Option<Object> = None;
            let mut name = String::new();
            let mut has_body = false;
            let mut fields = Vec::new();
            let mut parent: Option<Box<Object>> = None;
            let mut sort: Option<Sort> = None;

            let mut is_regular_line_break;

            let mut objects = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("//") {
                    continue;
                }
                if line.starts_with('(') {
                    has_body = line.ends_with('{');
                    let mut splits = line.split(")");
                    let sort_local: Sort = splits.next().unwrap().into();
                    sort = Some(sort_local);
                    let remaining = splits.next().unwrap();
                    let is_extension = remaining.contains("(:");
                    if is_extension {
                        let mut splits = remaining.split("(:");
                        name = splits.nth(0).unwrap().trim().to_string();
                        let parent_name = splits.next().unwrap();
                        let parent_name = parent_name.trim().to_string();
                        let parent_object =
                            objects.iter().find(|o: &&Object| o.name == parent_name);
                        if let Some(parent_object) = parent_object {
                            parent = Some(Box::new(parent_object.clone()));
                        } else {
                            assert!(false, "parent object with name {parent_name} not found");
                        }
                    } else {
                        let mut contents = remaining.split(':');
                        name = contents.next().unwrap().trim().to_string();
                        is_regular_line_break = contents.next().unwrap().ends_with("{");
                        if !is_regular_line_break {
                            assert!(false, "invalid formating");
                        }
                    }
                } else if !name.is_empty() && has_body {
                    if line.starts_with('}') {
                        let fields = fields.drain(..).collect();
                        objects.push(Object {
                            name: name.clone(),
                            fields,
                            parent: parent.clone(),
                            sort: {
                                if sort.is_some() {
                                    sort.clone().unwrap()
                                } else {
                                    Sort::Default
                                }
                            },
                            //TODO: add sparsed object find logic
                            has_sparsed_fields: false,
                        });
                        name = String::new();
                        parent = None;
                        sort = None;
                        // println!(".................");
                        // println!("{:?}", objects);
                        // println!(".................");
                        continue;
                    }
                    let mut splits = line.split(':');
                    let field_name = splits.next().unwrap().trim().to_string();
                    let field_type = splits.next().unwrap().trim().to_string();
                    fields.push(Field {
                        name: field_name,
                        type_: Type::from_str(&field_type),
                    });
                    // println!("{field_name} : {field_type}");
                } else if !has_body {
                    objects.push(Object {
                        name: name.clone(),
                        fields: vec![],
                        parent: parent.clone(),
                        sort: {
                            if sort.is_some() {
                                sort.clone().unwrap()
                            } else {
                                Sort::Default
                            }
                        },
                        //TODO: add sparsed object find logic
                        has_sparsed_fields: false,
                    });
                }
            }
            Ok(Schema {
                path: path.to_str().unwrap().to_string(),
                objects,
            })
        } else {
            Err(SchemeError::PathNotExists)
        }
    }
}
