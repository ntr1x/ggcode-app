use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::path::Path;
use clap::{arg, ArgMatches, Command};
use indoc::{formatdoc, indoc};
use prettytable::format::FormatBuilder;
use prettytable::{format, row, Table};
use ggcode_core::config::{Config, ScrollEntry};

use ggcode_core::ResolvedContext;
use ggcode_core::scroll::{Scroll, ScrollCommand};
use crate::config::{load_config, load_scroll, rm_scroll, save_config, save_scroll, save_string};

pub fn create_scroll_command() -> Command {
    Command::new("scroll")
        .about("Manage set of scrolls")
        .alias("g")
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

    let p = Path::new(path);

    assert!(!path.starts_with(".."), "The \"scroll\" directory should be nested within the project directory.");
    assert!(p.is_relative(), "The \"scroll\" directory should be defined relative to the project directory.");

    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();

    let p = &current_dir.join(path).canonicalize().unwrap();

    let registration = context.current_config.scrolls
        .iter()
        .find(|r| {
            match current_dir.join(&r.path).canonicalize().ok() {
                None => false,
                Some(rp) => rp.eq(p),
            }
        });

    let scroll_entries: Vec<ScrollEntry> = context.current_config.scrolls
        .iter()
        .filter(|r| {
            match current_dir.join(&r.path).canonicalize().ok() {
                None => true,
                Some(rp) => !rp.eq(p),
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

            save_config(&context.config_path, config)?;
            rm_scroll(path).unwrap();

            println!("Success!");
        }
    };

    Ok(())
}

fn execute_scroll_add_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let path = matches.get_one::<String>("path").unwrap();

    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();

    let p = Path::new(path);

    assert!(!path.starts_with(".."), "The \"scroll\" directory should be nested within the project directory.");
    assert!(p.is_relative(), "The \"scroll\" directory should be defined relative to the project directory.");

    let p = &current_dir.join(path).canonicalize().ok();

    let duplicate = context.current_config.scrolls
        .iter()
        .find(|r| {
            match current_dir.join(&r.path).canonicalize().ok() {
                None => false,
                Some(rp) => {
                    match p {
                        None => false,
                        Some(p) => rp.eq(p)
                    }
                },
            }
        });

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
                ScrollEntry { path: path.to_string() }
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

            save_string(&format!("{}/templates/README.md", path), readme.to_string())?;
            save_string(&format!("{}/variables/variables.yaml", path), variables.to_string())?;
            save_scroll(&format!("{}/ggcode-scroll.yaml", path), scroll)?;
            save_config(&context.config_path, config)?;

            println!("Success!");
        }
    }

    Ok(())
}

fn execute_scroll_list_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut scrolls: BTreeMap<String, Scroll> = BTreeMap::new();

    for repository in context.current_config.repositories.iter() {
        let repository_path = format!("ggcode_modules/{}", repository.name);
        let config_path = format!("{}/ggcode-info.yaml", repository_path);
        match load_config(config_path).ok() {
            None => {},
            Some(repository_config) => {
                for scroll_entry in repository_config.scrolls {
                    let scroll_config_path = format!("{}/{}/ggcode-scroll.yaml", repository_path, scroll_entry.path);
                    match load_scroll(scroll_config_path).ok() {
                        None => {},
                        Some(scroll) => {
                            scrolls.insert(format!("{}/{}", repository.name, scroll_entry.path), scroll);
                        },
                    }
                }
            },
        }
    }

    for scroll_entry in &context.current_config.scrolls {
        let scroll_config_path = format!("{}/ggcode-scroll.yaml", scroll_entry.path);
        match load_scroll(scroll_config_path).ok() {
            None => {},
            Some(scroll) => {
                scrolls.insert(format!("@/{}", scroll_entry.path), scroll);
            },
        }
    }

    let mut table = Table::new();

    let format = match matches.get_flag("condensed") {
        true => FormatBuilder::new().padding(0, 0).column_separator('\t').build(),
        false => *format::consts::FORMAT_BOX_CHARS
    };

    table.set_format(format);
    table.set_titles(row!["#", "Name"]);

    for (i, (name, scroll)) in scrolls.iter().enumerate() {
        table.add_row(row![
            format!("{}", i + 1).as_str(),
            name.as_str()
        ]);
    }

    table.printstd();
    Ok(())
}
