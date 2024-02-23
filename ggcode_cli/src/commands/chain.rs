use std::error::Error;

use clap::{arg, ArgMatches, Command};
use indoc::{formatdoc, indoc};
use prettytable::{format, row, Table};
use prettytable::format::FormatBuilder;
use relative_path::RelativePathBuf;

use ggcode_core::chain::{ChainCommand, ChainConfig, list_chains};
use ggcode_core::config::{ChainEntry, PackageConfig};
use ggcode_core::ResolvedContext;
use ggcode_core::storage::{resolve_inner_path, rm_chain, save_chain, save_config, save_string};
use crate::terminal::TerminalInput;

pub fn create_chain_command() -> Command {
    Command::new("chain")
        .about("Manage set of chains")
        .alias("c")
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(create_chain_list_command())
        .subcommand(create_chain_add_command())
        .subcommand(create_chain_remove_command())
}

fn create_chain_list_command() -> Command {
    Command::new("list")
        .about("List chains")
        .alias("ls")
        .arg(arg!(--condensed "Do not print table borders in output"))
}

fn create_chain_add_command() -> Command {
    Command::new("add")
        .about("Add a chain")
        .arg(arg!(-p --path <String> "Path to the chain directory"))
}

fn create_chain_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a chain")
        .alias("rm")
        .arg(arg!(-p --path <String> "Path to the chain directory"))
}

pub fn execute_chain_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => execute_chain_list_command(context, sub_matches),
        Some(("add", sub_matches)) => execute_chain_add_command(context, sub_matches),
        Some(("remove", sub_matches)) => execute_chain_remove_command(context, sub_matches),
        _ => unreachable!()
    }
}

fn execute_chain_remove_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let relative_path = TerminalInput::builder()
        .matches(matches)
        .name("path")
        .prompt("Relative inner path to a chain:")
        .required(true)
        .build()?
        .read(resolve_inner_path)?
        .unwrap();

    let registration = find_chain_with_name(context, &relative_path);

    let chain_entries: Vec<ChainEntry> = context.current_config.chains
        .iter()
        .filter(|r| {
            match resolve_inner_path(&r.path).ok() {
                None => true,
                Some(rp) => !relative_path.eq(&rp),
            }
        })
        .map(|e| e.clone())
        .collect();

    match registration {
        None => {},
        Some(_) => {
            let config = PackageConfig {
                chains: chain_entries,
                ..context.current_config.to_owned()
            };

            save_config(&resolve_inner_path(&context.config_path)?, config)?;
            rm_chain(&relative_path)
                .map_err::<Box<dyn Error>, _>(|e| format!("Cannot remove chain directory: {}", e).into())?;
        }
    };

    Ok(())
}

fn find_chain_with_name<'a>(context: &'a ResolvedContext, relative_path: &RelativePathBuf) -> Option<&'a ChainEntry> {
    context.current_config.chains
        .iter()
        .find(|r| {
            match resolve_inner_path(&r.path).ok() {
                None => false,
                Some(rp) => relative_path.eq(&rp),
            }
        })
}

fn execute_chain_add_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let relative_path = TerminalInput::builder()
        .matches(matches)
        .name("path")
        .prompt("Relative inner path to a chain:")
        .required(true)
        .build()?
        .read(resolve_inner_path)?
        .unwrap();

    let duplicate = find_chain_with_name(context, &relative_path);

    match duplicate {
        Some(_) => {
            println!("Skipped! (Duplicate)");
        },
        None => {
            let chain = ChainConfig {
                commands: vec![ChainCommand {
                    name: "default".to_string(),
                    about: Some("Default command".to_string()),
                    args: vec![],
                }],
            };

            let chain_entries = vec![
                ChainEntry { path: relative_path.to_string() }
            ];

            let config = PackageConfig {
                chains: [&context.current_config.chains[..], &chain_entries[..]].concat(),
                ..context.current_config.to_owned()
            };

            let readme = indoc!("
                # Generated content

                Author: {{ variables.author }}
                chain: {{ variables.chain }}
                Date: {{ now() }}
            ");

            let variables = formatdoc!("\
                author: \"{author}\"
                chain: \"{chain}\"
            ", author = "Developer", chain = relative_path.as_str());

            save_string(&relative_path.join("templates/README.md"), readme.to_string())?;
            save_string(&relative_path.join("variables/variables.yaml"), variables.to_string())?;
            save_chain(&relative_path.join("ggcode-chain.yaml"), chain)?;
            save_config(&resolve_inner_path(&context.config_path)?, config)?;
        }
    }

    Ok(())
}

fn execute_chain_list_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let chains = list_chains(context);

    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Path", "Alias", "Is Valid"]);

    for (i, (name, chain)) in chains.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            chain.chain_path,
            name.as_str(),
            match chain.chain {
                Some(_) => "valid",
                None => "invalid",
            }
        ]);
    }

    table.printstd();
    Ok(())
}
