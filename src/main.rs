mod api_map;
mod cli;
mod codegen;
mod conf;
mod process;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info, warn};
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use std::{fs, path::PathBuf, process::Command, fmt::format};

use crate::{cli::Cli, conf::Config, process::make_hl_ast};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config: Config = toml::from_str(
        fs::read_to_string(cli.config)
            .context("Failed to read the config file")?
            .as_str(),
    )?;

    _ = TermLogger::init(
        cli.verbose.log_level_filter(),
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    info!("Starting generation...");

    info!("Retrieving all functions...");
    let api_map_file_content = fs::read_to_string(cli.api_map)?;
    let api_map = api_map::parse(&api_map_file_content)?;

    debug!("Parsed&processed API map: {:#?}", api_map);

    info!("Generating HL-AST...");
    let hl_ast = make_hl_ast(api_map, &config);
    debug!("HL-AST: {hl_ast:#?}");

    // info!("Converting groups into AST...");

    // let mut namespaces_ast: Vec<Box<dyn Node>> = vec![];
    // for namespace in &namespaces_list.0 {
    //     let mut members: Vec<Box<dyn Node>> = vec![];
    //     for member in &namespace.members {
    //         let member = member.clone();
    //         members.push(Box::new(FunctionDeclaration {
    //             return_type: member.return_type,
    //             identifier: member.identifier,
    //             args: member.args,
    //             body: vec![],
    //         }));
    //     }

    //     namespaces_ast.push(Box::new(NamespaceDeclaration {
    //         identifier: namespace.identifier.clone(),
    //         members,
    //     }));
    // }

    // debug!("Resulting AST: {:#?}", namespaces_ast);
    // let namespace_ast = NamespaceDeclaration {
    //     identifier: "lvgl".to_string(),
    //     members: namespaces_ast,
    // };
    // debug!(
    //     "Codegen: {}",
    //     namespace_ast.gen_source(&config.generation.target)
    // );
    // let mut ast = vec![];
    // for namespace in &namespaces_ast {
    //     ast.push(namespace.gen_source(&config.generation.target));
    // }
    // debug!(
    //     "Codegen for first namespace: {}",
    //     namespaces_ast[0].gen_source(&conf::CxxVersion::Cxx20)
    // );
    // debug!("Generated source code: {}", namespaces_ast[0]);

    Ok(())
}
