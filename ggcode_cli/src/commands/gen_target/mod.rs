use std::error::Error;
use clap::{arg, ArgMatches, Command};
use prettytable::format::FormatBuilder;
use prettytable::{format, row, Table};
use ggcode_core::ResolvedContext;
use ggcode_core::config::{Config, TargetEntry};
use crate::config::{resolve_inner_path, save_config};

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
        .arg(arg!(-n --name <String> "Name of the target").required(true))
        .arg(arg!(-p --path <Path> "Target directory path").required(true))
        .arg_required_else_help(true)
}

fn create_target_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a target")
        .alias("rm")
        .arg(arg!(-n --name <String> "Name of the target").required(true))
        .arg_required_else_help(true)
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
    let name = matches.get_one::<String>("name").unwrap();

    let targets: Vec<TargetEntry> = context.current_config.targets
        .into_iter()
        .filter(|r| &r.name != name)
        .collect();

    let config = Config {
        targets,
        ..context.current_config
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_target_add_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = matches.get_one::<String>("name").unwrap();
    let path = matches.get_one::<String>("path").unwrap();

    println!("path: {}", path);

    let duplicate = context.current_config.targets
        .iter()
        .find(|r| r.name.eq(name));

    if duplicate.is_none() {
        let targets = vec![TargetEntry {
            name: name.to_string(),
            path: path.to_string(),
        }];
        let config = Config {
            targets: [&context.current_config.targets[..], &targets[..]].concat(),
            ..context.current_config
        };

        save_config(&resolve_inner_path(&context.config_path)?, config)?;
    }

    Ok(())
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
