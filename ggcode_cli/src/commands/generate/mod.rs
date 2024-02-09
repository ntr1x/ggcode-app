use std::error::Error;

use clap::{Arg, arg, ArgMatches, Command};
use console::style;
use relative_path::RelativePathBuf;

use ggcode_core::{Context, ResolvedContext};
use ggcode_core::scroll::{Scroll, ScrollCommand};

use crate::config::{load_templates, load_variables, resolve_package_path, resolve_target_path, save_target_file};
use crate::greetings::create_progress_bar;
use crate::renderer::Renderer;
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

pub fn create_generate_scroll_spell_command(_context: &ResolvedContext, _scroll_name: &String, _scroll: &Scroll, scroll_command: &ScrollCommand) -> Command {
    let mut subcommand = Command::new(&scroll_command.name)
        .about(scroll_command.about.clone().unwrap_or("Casting a magical spell".to_string()))
        .arg(arg!(-t --target <target> "The name of a well-known target").required(true))
        .arg_required_else_help(true);

    match scroll_command.args.len() {
        0 => {},
        _ => {
            // subcommand = subcommand.arg_required_else_help(true);
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

pub fn execute_generate_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some((name, sub_matches)) => execute_generate_scroll_command(context, &name.to_string(), sub_matches),
        _ => unreachable!()
    }
}

pub fn execute_generate_scroll_command(context: &ResolvedContext, name: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some((spell, sub_matches)) => execute_generate_scroll_spell_command(context, name, &spell.to_string(), sub_matches),
        _ => unreachable!()
    }
}

fn execute_generate_scroll_spell_command(context: &ResolvedContext, name: &String, _spell: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let target = matches.get_one::<String>("target").unwrap();

    let known_target = context.current_config.targets
        .iter()
        .find(|t| &t.name == target)
        .ok_or(format!("Unknown target: `{}`", style(target).red()).as_str())?;

    let target_path = resolve_target_path(&known_target.path)
        .expect("Cannot resolve target path");

    let path = resolve_package_path(name)?;
    let values_directory_path = path.join("variables");

    let variables = load_variables(&values_directory_path);

    let mut builder = Renderer::builder();
    for (key, value) in variables.as_mapping().unwrap() {
        builder = builder
            .with_value(key.as_str().unwrap().to_string(), value);
    }

    let templates_directory_path = path.join("templates");
    let templates = load_templates(templates_directory_path);

    for (key, value) in &templates {
        builder = builder
            .with_template_file(key, value)
            .with_template_string(key, key);
    }

    let renderer = builder.build()?;

    for (key, _value) in &templates {
        let pb = create_progress_bar();
        pb.set_message(format!("Generating file using `{}` template ...", key));
        let file_name = renderer.render(format!("raw:{}", key))?;
        let file_content = renderer.render(format!("path:{}", key))?;
        let file_relative_path = RelativePathBuf::from(file_name);

        save_target_file(&target_path, &file_relative_path, &file_content)?;

        let file_path = file_relative_path.to_path(&target_path).canonicalize().unwrap();
        pb.finish_with_message(format!("{} Generated file: {}", style("[DONE]").green(), file_path.to_str().unwrap().to_string()))
    }

    Ok(())
}
