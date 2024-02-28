use std::error::Error;

use clap::{arg, ArgAction, ArgMatches, Command};
use console::style;
use indoc::{formatdoc, indoc};
use prettytable::{format, row, Table};
use prettytable::format::FormatBuilder;
use relative_path::RelativePathBuf;

use ggcode_core::config::{PackageConfig, ScrollEntry};
use ggcode_core::ResolvedContext;
use ggcode_core::scroll::{find_scroll_by_name, list_scrolls};
use ggcode_core::storage::{resolve_inner_path, rm_scroll, save_config, save_string};

use crate::terminal::flag::TerminalFlag;
use crate::terminal::input::TerminalInput;

pub fn create_scroll_command() -> Command {
    Command::new("scroll")
        .about("Manage set of scrolls")
        .alias("s")
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(create_scroll_list_command())
        .subcommand(create_scroll_add_command())
        .subcommand(create_scroll_remove_command())
}

fn create_scroll_list_command() -> Command {
    Command::new("list")
        .about("List scrolls")
        .alias("ls")
        .arg(arg!(--condensed "Do not print table borders in output"))
}

fn create_scroll_add_command() -> Command {
    Command::new("add")
        .about("Add a scroll")
        .arg(arg!(-n --name <String> "Name of the scroll"))
        .arg(arg!(-p --path <String> "Path to the scroll directory"))
}

fn create_scroll_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a scroll")
        .alias("rm")
        .arg(arg!(-n --name <String> "Name of the scroll"))
        .arg(arg!(-f --force "Force removal of the scroll directory").action(ArgAction::Set))
}

pub fn execute_scroll_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => execute_scroll_list_command(context, sub_matches),
        Some(("add", sub_matches)) => execute_scroll_add_command(context, sub_matches),
        Some(("remove", sub_matches)) => execute_scroll_remove_command(context, sub_matches),
        _ => unreachable!()
    }
}

fn execute_scroll_remove_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the scroll to remove:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let force = TerminalFlag::builder()
        .matches(matches)
        .name("force")
        .prompt("Remove also a scroll directory?")
        .required(true)
        .default_value(false)
        .build()?
        .read_bool()?
        .unwrap();

    let mut scrolls: Vec<ScrollEntry> = vec![];

    let mut scroll_to_delete: Option<ScrollEntry> = None;

    for scroll in &context.current_config.scrolls {
        if &scroll.name != &name {
            scrolls.push(scroll.clone())
        } else {
            scroll_to_delete = Some(scroll.clone());
        }
    }

    if &scrolls.len() == &context.current_config.scrolls.len() {
        return Err(format!("No scroll with name: {}. Nothing changed.", name).into());
    }

    let config = PackageConfig {
        scrolls,
        ..context.current_config.to_owned()
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    if let Some(scroll) = scroll_to_delete {
        let relative_path = RelativePathBuf::from(scroll.path);
        if force {
            rm_scroll(&relative_path)
                .map_err::<Box<dyn Error>, _>(|e| format!("Cannot remove scroll directory: {}", e).into())?;
        } else {
            eprintln!("{} Configuration entry has been unregistered, but the data in directory {} will not be deleted automatically. Please delete it manually if needed.", style("[INFO]").yellow(), relative_path);
        }
    }

    Ok(())
}

fn execute_scroll_add_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("Name of the scroll:")
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let relative_path = TerminalInput::builder()
        .matches(matches)
        .name("path")
        .prompt("Relative inner path to the scroll directory:")
        .required(true)
        .build()?
        .read(resolve_inner_path)?
        .unwrap();

    let duplicate = find_scroll_by_name(&context, &context.current_config, &name);

    if duplicate.is_some() {
        return Err(format!("Duplicate name: {}. Nothing changed.", name).into());
    }

    let scroll_entries = vec![
        ScrollEntry { name, path: relative_path.to_string(), about: None }
    ];

    let config = PackageConfig {
        scrolls: [&context.current_config.scrolls[..], &scroll_entries[..]].concat(),
        ..context.current_config.to_owned()
    };

    let readme = indoc!("
        # Generated content

        Author: {{ variables.author }}
        Scroll: {{ variables.scroll }}
        Date: {{ now() }}
    ");

    let variables = formatdoc!("\
        author: \"{author}\"
        scroll: \"{scroll}\"
    ", author = "Developer", scroll = relative_path.as_str());

    save_string(&relative_path.join("templates/README.md.tera"), readme.to_string())?;
    save_string(&relative_path.join("variables/variables.yaml"), variables.to_string())?;
    save_config(&resolve_inner_path(&context.config_path)?, config)?;

    Ok(())
}

fn execute_scroll_list_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let scrolls = list_scrolls(context);

    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Alias", "Package", "Path"]);

    for (i, (_, scroll)) in scrolls.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            scroll.full_name,
            scroll.package.name,
            scroll.scroll.path
        ]);
    }

    table.printstd();
    Ok(())
}
