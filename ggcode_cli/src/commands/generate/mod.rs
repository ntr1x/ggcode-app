use std::error::Error;

use clap::{Arg, ArgMatches, Command};

use ggcode_core::{Context, ResolvedContext};
use ggcode_core::scroll::{Scroll, ScrollCommand};

use crate::structure::list_scrolls;

pub fn create_generate_command(context: &Context) -> Command {
    let mut command = Command::new("generate")
        .about("Execute generation script from scroll")
        .alias("g")
        .arg_required_else_help(true);

    match context.resolve() {
        Ok(resolved_context) => {
            let scrolls = list_scrolls(&resolved_context);

            for (name, scroll) in scrolls {
                if let Some(scroll) = scroll {
                    let subcommand = create_generate_scroll_command(&resolved_context, &name, &scroll);
                    command = command.subcommand(subcommand);
                }
            }
        },
        Err(_e) => {},
    }

    command
}

pub fn create_generate_scroll_command(context: &ResolvedContext, scroll_name: &String, scroll: &Scroll) -> Command {
    let mut command = Command::new(scroll_name.clone())
        .about("Casting spells from a magical scroll")
        .arg_required_else_help(true);

    match scroll.commands.len() {
        0 => {
            let subcommand = create_generate_scroll_spell_command(context, scroll_name, scroll, &ScrollCommand {
                name: "default".to_string(),
                about: None,
                args: vec![]
            });
            command = command.subcommand(subcommand);
        },
        _ => {
            for command_entry in &scroll.commands {
                let subcommand = create_generate_scroll_spell_command(context, scroll_name, scroll, command_entry);
                command = command.subcommand(subcommand);
            }
        }
    };

    command
}

pub fn create_generate_scroll_spell_command(context: &ResolvedContext, scroll_name: &String, scroll: &Scroll, scroll_command: &ScrollCommand) -> Command {
    let mut subcommand = Command::new(&scroll_command.name)
        .about(scroll_command.about.clone().unwrap_or("Casting a magical spell".to_string()));

    match scroll_command.args.len() {
        0 => {},
        _ => {
            subcommand = subcommand.arg_required_else_help(true);
            for arg_entry in &scroll_command.args {
                let arg = Arg::new(&arg_entry.name)
                    .long(&arg_entry.name)
                    .help(arg_entry.about.clone().unwrap_or("Magical spell option".to_string()))
                    .required(arg_entry.required.unwrap_or(false));
                subcommand = subcommand.arg(arg);
            }
        }
    };

    subcommand
}

pub fn execute_generate_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some((name, sub_matches)) => execute_generate_do_command(context, sub_matches),
        _ => unreachable!()
    }
}

fn execute_generate_do_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        Ok(())
}
