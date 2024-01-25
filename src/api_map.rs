use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use log::error;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct JSONRoot {
    pub enums: Vec<JSONValue>,
    pub functions: Vec<JSONValue>,
    pub structures: Vec<JSONValue>,
    pub unions: Vec<JSONValue>,
    pub variables: Vec<JSONValue>,
    pub typedefs: Vec<JSONValue>,
    pub forward_decls: Vec<JSONValue>,
    pub macros: Vec<JSONValue>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct JSONValue {
    pub name: Option<String>,
    pub json_type: JSONType,
    pub docstring: Option<String>,
    pub quals: Option<Vec<String>>,
    pub storage: Option<Vec<String>>,
    pub r#type: Option<Box<JSONValue>>, // wrapped in a Box to fix type recursion
    pub fields: Option<Vec<JSONValue>>,
    pub members: Option<Vec<JSONValue>>,
    pub args: Option<Vec<JSONValue>>,
    pub bitsize: Option<String>,
}

impl JSONValue {
    fn parse_as_type(&self) -> String {
        if self.r#type.clone().unwrap().name.is_none() {
            return format!("{}*", self.r#type.clone().unwrap().parse_as_type());
        }

        return self.r#type.clone().unwrap().name.unwrap();
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JSONType {
    PrimitiveType,
    StdlibType,
    LvglType,
    EnumMember,
    Field,
    Struct,
    Typedef,
    Enum,
    Function,
    Pointer,
    Array,
    #[serde(rename = "ret_type")]
    ReturnType,
    FunctionPointer,
    Variable,
    Union,
    #[serde(rename = "forward_decl")]
    ForwardDeclaration,
    Macro,
    Arg,
    SpecialType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct APIMap {
    pub enums: Vec<Enum>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
}

#[derive(Debug, Clone, PartialEq)]
struct Typedefs {
    pub named: HashMap<String, String>,
    pub unnamed: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub identifier: String,
    pub r#type: String,
    pub members: Vec<(String, Option<String>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub identifier: String,
    pub return_type: String,
    pub args: Vec<FuncArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncArg {
    pub identifier: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub identifier: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub identifier: String,
    pub kind: String,
    pub bitsize: Option<u8>,
}

// god forgive me for the amount of .unwrap() statements that
// there will be in this function
pub fn parse(source_str: &str) -> Result<APIMap> {
    let json: JSONRoot = serde_json::from_str(source_str)?;

    let Typedefs = json.process_typedefs();

    Ok(APIMap {
        enums: json.process_enums(),
        functions: json.process_functions(),
        structs: json.process_structs(),
    })
}

impl JSONRoot {
    fn process_enums(&self) -> Vec<Enum> {
        self.enums
            .clone()
            .into_iter()
            .map(|item| {
                Enum {
                    identifier: item.name.unwrap_or("anonymous".to_string()),
                    members: item
                        .members
                        .unwrap()
                        .into_iter()
                        .map(|member| {
                            // Always None because the JSON doesn't have
                            // enum member value parsing, sadly
                            (member.name.unwrap(), None::<String>)
                        })
                        .collect(),
                    r#type: item.r#type.unwrap().name.unwrap(),
                }
            })
            .collect()
    }

    fn process_functions(&self) -> Vec<Function> {
        self.functions
            .clone()
            .into_iter()
            .map(|func| {
                let func = func.clone();
                Function {
                    identifier: func.name.clone().unwrap(),
                    return_type: func.r#type.clone().unwrap().parse_as_type(),
                    args: func
                        .clone()
                        .args
                        .unwrap_or(vec![])
                        .into_iter()
                        .map(|arg| FuncArg {
                            identifier: arg.name.clone(),
                            kind: arg.parse_as_type(),
                        })
                        .filter(|arg| {
                            if func.args.clone().unwrap_or(vec![]).len() == 1
                                && arg.identifier.is_none()
                                && arg.kind == "void"
                            {
                                return false;
                            }
                            true
                        })
                        .collect(),
                }
            })
            .collect()
    }

    fn process_structs(&self) -> Vec<Struct> {
        self.structures
            .clone()
            .into_iter()
            .map(|structure| {
                let s = Struct {
                    identifier: structure.name.clone().unwrap(),
                    fields: structure
                        .fields
                        .unwrap()
                        .into_iter()
                        .map(|field| StructField {
                            identifier: field.clone().name.unwrap(),
                            kind: field.parse_as_type(),
                            bitsize: field.bitsize.map(|x| x.parse().unwrap()),
                        })
                        .collect(),
                };
                println!("{}", structure.name.unwrap());

                s
            })
            .collect()
    }

    fn process_typedefs(&self) -> Typedefs {
        let mut named_typedefs: HashMap<String, String> = HashMap::new();

        let mut unnamed_typedefs: Vec<(String, String)> = vec![];

        for typedef in &self.typedefs.clone() {
            let typedef = typedef.clone();

            let kind = typedef.parse_as_type();

            // TODO : Refactor this
            if kind.contains("struct")
                || kind.contains("int*")
                || kind.contains("uint8_t")
                || kind.contains("uint16_t")
                || kind.contains("uint32_t")
                || kind.contains("float")
                || kind.contains("double")
                || kind.contains("uintptr_t")
                || kind.contains("intptr_t")
                || kind.contains("void*")
                || kind.contains("int")
                || kind.contains("union")
                || kind.contains("int8_t")
            {
                unnamed_typedefs.push((kind, typedef.name.unwrap()));
            } else {
                if named_typedefs.insert(kind.clone(), typedef.name.clone().unwrap()).is_some() {
                    error!("Duplicate typedef: {} -> {}", typedef.name.unwrap(), kind);
                }
            }
        }

        println!("{:?}", named_typedefs);
        println!("{:?}", unnamed_typedefs);

        Typedefs {
            named: named_typedefs,
            unnamed: unnamed_typedefs,
        }
    }
}
