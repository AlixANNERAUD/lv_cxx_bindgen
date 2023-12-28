use std::fs;

use anyhow::Result;
use log::debug;
use tree_sitter::{Node, Parser, Tree};

#[derive(Debug)]
pub struct Function {
    pub return_type: String,
    pub identifier: String,
    pub args: Vec<TypedValue>,
}

#[derive(Debug)]
pub struct TypedValue {
    pub identifier: Option<String>,
    // type
    pub kind: String,
}

pub fn headers(input: &[String]) -> Result<Vec<Function>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_cpp::language())?;
    let mut functions = vec![];

    for path in input {
        let file_str = fs::read_to_string(path)?;
        let tree = parser.parse(&file_str, None).unwrap();
        let root = tree.root_node();

        let function = walk_node(root, &file_str);

        functions.push(function);
    }

    return Ok(functions.into_iter().flatten().collect());
}

fn walk_node(node: Node, source_str: &str) -> Vec<Function> {
    let mut functions = vec![];
    for i in 0..node.named_child_count() {
        let child = node.named_child(i).unwrap();
        debug!("{:?}", child.kind());

        if child.kind() == "preproc_ifdef"
            || child.kind() == "linkage_specification"
            || child.kind() == "declaration_list"
        {
            debug!("Walking into new node");
            functions.append(&mut walk_node(child, source_str));
        }

        if child.kind() == "declaration" {
            debug!("Found declaration");
            let function = parse_function_declaration(child, source_str).unwrap();
            debug!("Parsed function: {:#?}", function);
            functions.push(function);
        }
    }
    return functions;
}

fn parse_function_declaration(node: Node, source_str: &str) -> Option<Function> {
    let type_range = node.named_child(0)?.range();
    let type_str = &source_str[type_range.start_byte..type_range.end_byte];
    debug!("Type: {:?}", type_str);

    let declarator_node = node.child_by_field_name("declarator").unwrap();
    let function_name_node = declarator_node.child_by_field_name("declarator").unwrap();
    let function_name_str =
        &source_str[function_name_node.range().start_byte..function_name_node.range().end_byte];
    debug!("Function name: {:?}", function_name_str);

    let parameters_node = declarator_node.child_by_field_name("parameters").unwrap();
    let mut parameters = vec![];
    debug!(
        "Parameters node: {}, {}, {}",
        parameters_node.to_sexp(),
        &source_str[parameters_node.range().start_byte..parameters_node.range().end_byte],
        parameters_node.child_count()
    );

    for i in 1..parameters_node.child_count() - 1 {
        debug!("At parameter node {}", i);
        let parameter_declaration = parameters_node.child(i).unwrap();
        debug!(
            "Parameter declaration: {} {}",
            parameter_declaration.to_sexp(),
            &source_str
                [parameter_declaration.range().start_byte..parameter_declaration.range().end_byte]
        );
        let parameter_type = parameter_declaration.child(0).unwrap();
        let parameter_identifier = parameter_declaration.child(1);
        let type_str =
            &source_str[parameter_type.range().start_byte..parameter_type.range().end_byte];
        let mut identifier_str = None;
        if let Some(ident) = parameter_identifier {
            identifier_str =
                Some(source_str[ident.range().start_byte..ident.range().end_byte].to_string());
            debug!("Identifier exists: {:?}", identifier_str);
        }

        let typed_value = TypedValue {
            identifier: identifier_str,
            kind: type_str.to_string(),
        };
        debug!("Resulting parameter: {:?}", typed_value);

        parameters.push(typed_value);
    }

    debug!("Parameters: {:?}", parameters);

    return Some(Function {
        identifier: function_name_str.to_string(),
        args: parameters,
        return_type: type_str.to_string()
    });
}
