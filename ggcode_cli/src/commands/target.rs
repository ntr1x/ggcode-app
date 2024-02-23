use std::error::Error;

use clap::{arg, ArgMatches, Command, ValueHint};
use console::style;
use prettytable::{format, row, Table};
use prettytable::format::FormatBuilder;

use ggcode_core::config::{PackageConfig, TargetEntry};
use ggcode_core::ResolvedContext;

use crate::storage::{resolve_inner_path, resolve_target_path, save_config};
use crate::terminal::TerminalInput;

pub fn create_target_command() -> Command {
    Command::new("target")
        .about("Manage set of targets")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(create_target_add_command())
        .subcommand(create_target_remove_command())
        .subcommand(create_target_list_command())
}

fn create_target_add_command() -> Command {
    Command::new("add")
        .about("Add a target")
        .arg(arg!(-n --name <String> "Name of the target"))
        .arg(arg!(-p --path <Path> "Target directory path").value_hint(ValueHint::DirPath))
}

fn create_target_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a target")
        .alias("rm")
        .arg(arg!(-n --name <String> "Name of the target"))
}

fn create_target_list_command() -> Command {
    Command::new("list")
        .about("List targets")
        .alias("ls")
        .arg(arg!(--condensed "Do not print table borders in output"))
}

pub fn execute_target_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => execute_target_list_command(context, sub_matches),
        Some(("add", sub_matches)) => execute_target_add_command(context, sub_matches),
        Some(("remove", sub_matches)) => execute_target_remove_command(context, sub_matches),
        _ => unreachable!()
    }
}

fn execute_target_remove_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the target:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let mut targets: Vec<TargetEntry> = vec![];

    for target in &context.current_config.targets {
        if &target.name != &name {
            targets.push(target.clone())
        }
    }

    if &targets.len() == &context.current_config.targets.len() {
        eprintln!("{} Nothing changed. No target with name: {}", style("[WARN]").yellow(), name)
    }

    let config = PackageConfig {
        targets,
        ..context.current_config
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_target_add_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the target:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    // let path = matches.get_one::<String>("path").unwrap();
    let path = TerminalInput::builder()
        .matches(matches)
        .name("path")
        .prompt("Path to the target:")
        .required(true)
        .build()?
        .read(resolve_target_path)?
        .unwrap();

    let duplicate = context.current_config.targets
        .iter()
        .find(|r| r.name.eq(&name));

    match duplicate {
        Some(_) => {
            Err(format!("Target {} already exists", name).into())
        }
        None => {
            let targets = vec![TargetEntry {
                name: name.to_string(),
                path: path.as_path().to_str().unwrap().to_string()
            }];
            let config = PackageConfig {
                targets: [&context.current_config.targets[..], &targets[..]].concat(),
                ..context.current_config
            };

            save_config(&resolve_inner_path(&context.config_path)?, config)?;
            Ok(())
        }
    }
}

fn execute_target_list_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Name", "Path"]);

    for (i, target) in context.current_config.targets.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            target.name.as_str(),
            target.path.as_str()
        ]);
    }

    table.printstd();
    Ok(())
}
