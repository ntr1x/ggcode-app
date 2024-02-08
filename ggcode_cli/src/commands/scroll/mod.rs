use std::error::Error;
use clap::{arg, ArgMatches, Command};
use indoc::{formatdoc, indoc};
use prettytable::format::FormatBuilder;
use prettytable::{format, row, Table};
use relative_path::RelativePathBuf;
use ggcode_core::config::{Config, ScrollEntry};

use ggcode_core::ResolvedContext;
use ggcode_core::scroll::{Scroll, ScrollCommand};
use crate::config::{load_scroll, resolve_inner_path, rm_scroll, save_config, save_scroll, save_string};
use crate::structure::list_scrolls;

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
        .arg(arg!(-p --path <String> "Path to the scroll directory").required(true))
        .arg_required_else_help(true)
}

fn create_scroll_remove_command() -> Command {
    Command::new("remove")
        .about("Remove a scroll")
        .alias("rm")
        .arg(arg!(-p --path <String> "Path to the scroll directory").required(true))
        .arg_required_else_help(true)
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
    let path = matches.get_one::<String>("path").unwrap();
    let relative_path = resolve_inner_path(path)?;

    let registration = find_scroll_with_name(context, &relative_path);

    let scroll_entries: Vec<ScrollEntry> = context.current_config.scrolls
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
            let config = Config {
                scrolls: scroll_entries,
                ..context.current_config.to_owned()
            };

            save_config(&resolve_inner_path(&context.config_path)?, config)?;
            rm_scroll(&resolve_inner_path(path)?).unwrap();

            println!("Success!");
        }
    };

    Ok(())
}

fn find_scroll_with_name<'a>(context: &'a ResolvedContext, relative_path: &RelativePathBuf) -> Option<&'a ScrollEntry> {
    context.current_config.scrolls
        .iter()
        .find(|r| {
            match resolve_inner_path(&r.path).ok() {
                None => false,
                Some(rp) => relative_path.eq(&rp),
            }
        })
}

fn execute_scroll_add_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let path = matches.get_one::<String>("path").unwrap();
    let relative_path = resolve_inner_path(path)?;

    let duplicate = find_scroll_with_name(context, &relative_path);

    match duplicate {
        Some(_) => {
            println!("Skipped! (Duplicate)");
        },
        None => {
            let scroll = Scroll {
                commands: vec![ScrollCommand {
                    name: "default".to_string(),
                    about: Some("Default command".to_string()),
                    args: vec![],
                }],
            };

            let scroll_entries = vec![
                ScrollEntry { path: relative_path.to_string() }
            ];

            let config = Config {
                scrolls: [&context.current_config.scrolls[..], &scroll_entries[..]].concat(),
                ..context.current_config.to_owned()
            };

            let readme = indoc!("
                # Generated content

                Author: {{ variables.author }}
                Scroll: {{ variables.scroll }}
                Date: {{ now() }}
            ");

            let variables = formatdoc!("
                \"[README.md]\":
                  author: \"{author}\"
                  scroll: \"{scroll}\"
            ", author = "Developer", scroll = path);


            save_string(&relative_path.join("templates/README.md"), readme.to_string())?;
            save_string(&relative_path.join("variables/variables.yaml"), variables.to_string())?;
            save_scroll(&relative_path.join("ggcode-scroll.yaml"), scroll)?;
            save_config(&resolve_inner_path(&context.config_path)?, config)?;

            println!("Success!");
        }
    }

    Ok(())
}

fn execute_scroll_list_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut scrolls = list_scrolls(context);

    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Name", "Is Valid"]);

    for (i, (name, scroll)) in scrolls.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            name.as_str(),
            match scroll {
                Some(_) => "valid",
                None => "invalid",
            }
        ]);
    }

    table.printstd();
    Ok(())
}
