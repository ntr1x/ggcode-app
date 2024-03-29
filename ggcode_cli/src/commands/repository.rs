use std::error::Error;

use clap::{arg, ArgMatches, Command};
use console::style;
use prettytable::{format, row, Table};
use prettytable::format::FormatBuilder;

use ggcode_core::config::{PackageConfig, RepositoryEntry};
use ggcode_core::ResolvedContext;
use ggcode_core::storage::{resolve_inner_path, save_config};
use crate::terminal::input::TerminalInput;

pub fn create_repository_command() -> Command {
    Command::new("repository")
        .about("Manage set of repositories")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(create_repository_add_command())
        .subcommand(create_repository_remove_command())
        .subcommand(create_repository_list_command())
}

fn create_repository_add_command() -> Command {
    Command::new("add")
        .about("Add a repository")
        .arg(arg!(-n --name <String> "Name of the repository"))
        .arg(arg!(-u --uri <URI> "URI of the repository"))
}

fn create_repository_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a repository")
        .alias("rm")
        .arg(arg!(-n --name <String> "Name of the repository"))
}

fn create_repository_list_command() -> Command {
    Command::new("list")
        .about("List repositories")
        .alias("ls")
        .arg(arg!(--condensed "Do not print table borders in output"))
}

pub fn execute_repository_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => execute_repository_list_command(context, sub_matches),
        Some(("add", sub_matches)) => execute_repository_add_command(context, sub_matches),
        Some(("remove", sub_matches)) => execute_repository_remove_command(context, sub_matches),
        Some((other, _)) => return Err(format!("Unsupported command: {}", other).into()),
        _ => unreachable!()
    }
}

fn execute_repository_remove_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the repository:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let mut repositories: Vec<RepositoryEntry> = vec![];

    for repository in &context.current_config.repositories {
        if &repository.name != &name {
            repositories.push(repository.clone())
        }
    }

    if &repositories.len() == &context.current_config.repositories.len() {
        eprintln!("{} Nothing changed. No repository with name: {}", style("[WARN]").yellow(), name)
    }

    let config = PackageConfig {
        repositories,
        ..context.current_config
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_repository_add_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the repository:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let uri = TerminalInput::builder()
        .matches(matches)
        .name("uri")
        .prompt("URI of the repository:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let duplicate = context.current_config.repositories
        .iter()
        .find(|r| r.name.eq(&name));

    if duplicate.is_none() {
        let repositories = vec![RepositoryEntry {
            name: name.to_string(),
            uri: uri.to_string(),
        }];
        let config = PackageConfig {
            repositories: [&context.current_config.repositories[..], &repositories[..]].concat(),
            ..context.current_config
        };

        save_config(&resolve_inner_path(&context.config_path)?, config)?;
    }

    Ok(())
}

fn execute_repository_list_command(context: ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Name", "URI"]);

    for (i, repository) in context.current_config.repositories.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            repository.name.as_str(),
            repository.uri.as_str()
        ]);
    }

    table.printstd();
    Ok(())
}
