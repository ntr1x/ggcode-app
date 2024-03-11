use std::error::Error;

use clap::{arg, ArgMatches, Command};
use indoc::indoc;
use prettytable::{format, row, Table};
use prettytable::format::FormatBuilder;

use ggcode_core::action::{find_action_by_name, list_actions};
use ggcode_core::config::{ActionEntry, PackageConfig};
use ggcode_core::ResolvedContext;
use ggcode_core::storage::{resolve_inner_path, save_config, save_string};

use crate::terminal::input::TerminalInput;

pub fn create_action_command() -> Command {
    Command::new("action")
        .about("Manage set of runnable actions")
        .alias("c")
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(create_action_list_command())
        .subcommand(create_action_add_command())
        .subcommand(create_action_remove_command())
}

fn create_action_list_command() -> Command {
    Command::new("list")
        .about("List runnable actions")
        .alias("ls")
        .arg(arg!(--condensed "Do not print table borders in output"))
}

fn create_action_add_command() -> Command {
    Command::new("add")
        .about("Add an action")
        .arg(arg!(-n --name <String> "Name of the runnable action"))
        .arg(arg!(-p --path <String> "Path to the runnable action file"))
}

fn create_action_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a runnable action")
        .alias("rm")
        .arg(arg!(-n --name <String> "Name of the runnable action"))
}

pub fn execute_action_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => execute_action_list_command(context, sub_matches),
        Some(("add", sub_matches)) => execute_action_add_command(context, sub_matches),
        Some(("remove", sub_matches)) => execute_action_remove_command(context, sub_matches),
        Some((other, _)) => return Err(format!("Unsupported command: {}", other).into()),
        _ => unreachable!()
    }
}

fn execute_action_remove_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the runnable action:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let mut actions: Vec<ActionEntry> = vec![];

    for action in &context.current_config.actions {
        if &action.name != &name {
            actions.push(action.clone())
        }
    }

    if &actions.len() == &context.current_config.actions.len() {
        return Err(format!("No runnable action with name: {}. Nothing changed.", name).into());
    }

    let config = PackageConfig {
        actions,
        ..context.current_config.to_owned()
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_action_add_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the runnable action")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let relative_path = TerminalInput::builder()
        .matches(matches)
        .name("path")
        .prompt("Relative inner path to the runnable action file")
        .required(true)
        .build()?
        .read(resolve_inner_path)?
        .unwrap();

    let relative_path = match &relative_path.extension() {
        None => relative_path.with_extension("luau"),
        Some(ext) => match *ext == "luau" {
            true => relative_path,
            false => return Err(format!("Invalid extension: {}", ext).into())
        }
    };

    let duplicate = find_action_by_name(context, &context.current_config, &name);

    if duplicate.is_some() {
        return Err(format!("Duplicate name: {}. Nothing changed.", name).into());
    }

    let action_entries = vec![
        ActionEntry {
            name: name.clone(),
            path: relative_path.to_string(),
            about: None,
            args: vec![]
        }
    ];

    let config = PackageConfig {
        actions: [&context.current_config.actions[..], &action_entries[..]].concat(),
        ..context.current_config.to_owned()
    };

    let run_luau = indoc!("
        print('Executing...')
        if args and args.verbose then
            print('Trace info')
        end
        print('Done!')
    ");

    save_string(&relative_path, run_luau.to_string())?;
    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_action_list_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let commands = list_actions(context);

    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Name", "Path"]);

    for (i, (name, command)) in commands.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            name.as_str(),
            command.action.path,
        ]);
    }

    table.printstd();
    Ok(())
}
