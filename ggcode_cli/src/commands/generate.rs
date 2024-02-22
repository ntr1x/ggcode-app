use std::error::Error;

use clap::{Arg, arg, ArgMatches, Command};
use console::style;
use convert_case::{Case, Casing};
use relative_path::RelativePathBuf;
use serde_yaml::{Mapping, Value};

use ggcode_core::{Context, ResolvedContext};
use ggcode_core::scroll::{Scroll, ScrollCommand};

use crate::storage::{load_scroll, load_templates, load_variables, resolve_package_path, resolve_target_path, save_target_file};
use crate::greetings::create_progress_bar;
use crate::renderer::builder::RendererBuilder;
use crate::structure::list_scrolls;
use crate::utils::evaluate;

pub fn create_generate_command(context: &Context) -> Result<Command, Box<dyn Error>> {
    let mut command = Command::new("generate")
        .about("Execute generation script from scroll")
        .alias("g")
        .arg_required_else_help(true);

    if let Some(resolved_context) = context.resolve().ok() {
        let scrolls = list_scrolls(&resolved_context);
        for (name, scroll) in scrolls {
            if let Some(scroll) = scroll.scroll {
                let subcommand = create_generate_scroll_command(&resolved_context, &name, &scroll)?;
                command = command.subcommand(subcommand);
            }
        }
    }

    Ok(command)
}

pub fn create_generate_scroll_command(
    context: &ResolvedContext,
    scroll_name: &String,
    scroll: &Scroll
) -> Result<Command, Box<dyn Error>> {
    let mut command = Command::new(scroll_name.clone())
        .about("Casting spells from a magical scroll")
        .arg_required_else_help(true);

    let default_commands = vec![
        ScrollCommand { name: "default".to_string(), about: None, args: vec![] }
    ];

    let command_vec = match scroll.commands.len() {
        0 => &default_commands,
        _ => &scroll.commands
    };

    for command_entry in command_vec {
        let subcommand = create_generate_scroll_spell_command(context, scroll_name, scroll, command_entry)?;
        command = command.subcommand(subcommand);
    }

    Ok(command)
}

pub fn create_generate_scroll_spell_command(
    _context: &ResolvedContext,
    _scroll_name: &String,
    _scroll: &Scroll,
    scroll_command: &ScrollCommand
) -> Result<Command, Box<dyn Error>> {
    let mut subcommand = Command::new(&scroll_command.name)
        .about(scroll_command.about.clone().unwrap_or("Casting a magical spell".to_string()))
        .arg(arg!(-t --target <target> "The name of a well-known target")
            .required_unless_present("target-path"))
        .arg(Arg::new("target-path")
            .long("target-path")
            .short('p')
            .help("The path to output directory")
            .required_unless_present("target"))
        .arg(arg!(-v --variables <path> "Path to a file or directory containing variable overrides").required(false))
        .arg(Arg::new("dry-run")
            .long("dry-run".to_string())
            .short('d')
            .num_args(0)
            .help("Do not generate files; simply test the ability to render templates"))
        .arg_required_else_help(true);

    for arg_entry in &scroll_command.args {
        match arg_entry.name.as_str() {
            "target" | "target-path" | "variables" | "dry-run" | "help" =>
                return Err(format!("Invalid argument: {}", &arg_entry.name).into()),
            _ => {}
        }
        let arg = Arg::new(&arg_entry.name)
            .long(&arg_entry.name)
            .help(arg_entry.about.clone().unwrap_or("Magical spell option".to_string()))
            .required(arg_entry.required.unwrap_or(false));
        subcommand = subcommand.arg(arg);
    }

    Ok(subcommand)
}

pub fn execute_generate_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some((path, sub_matches)) => execute_generate_scroll_command(context, &path.to_string(), sub_matches),
        _ => unreachable!()
    }
}

pub fn execute_generate_scroll_command(context: &ResolvedContext, name: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let scroll_path = resolve_package_path(&name)?;
    let scroll_config_path = scroll_path.join("ggcode-scroll.yaml");
    let scroll = load_scroll(&scroll_config_path)?;

    match matches.subcommand() {
        Some((spell_name, sub_matches)) => execute_generate_scroll_spell_command(context, &name, &scroll, &spell_name.to_string(), sub_matches),
        _ => unreachable!()
    }
}

fn execute_generate_scroll_spell_command(context: &ResolvedContext, name: &String, scroll: &Scroll, spell_name: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let spell = scroll.commands
        .iter()
        .find(|c| &c.name == spell_name)
        .ok_or::<Box<dyn Error>>(format!("Invalid usage. Unknown spell: {}", style(spell_name).yellow()).as_str().into())?;

    let target_name = matches.get_one::<String>("target");
    let target_path = matches.get_one::<String>("target-path");
    let overrides_path = matches.try_get_one::<String>("variables")
        .map_err::<Box<dyn Error>, _>(|e| e.into())
        .map(|path| match path {
            Some(p) => Some(resolve_target_path(p).ok()?),
            None => None
        })?;

    let dry_run = matches.get_one::<bool>("dry-run").unwrap();

    if let Some(path) = &overrides_path {
        eprintln!("overrides: {}", path.to_str().unwrap());
    }

    let resolved_target_path = match (target_path, target_name) {
        (None, None) => return Err("Invalid usage. Specify `--target` or `--target-path` option.".into()),
        (None, Some(name)) => {
            let target = context.current_config.targets
                .iter()
                .find(|t| &t.name == name)
                .ok_or::<Box<dyn Error>>(format!("Invalid usage. Unknown target: {}", style(name).yellow()).as_str().into())?;
            resolve_target_path(&target.path)?
        },
        (Some(path), None) => resolve_target_path(&path)?,
        (Some(_), Some(_)) => return Err(format!(
            "Invalid usage. The options {target} and {target_path} should not be used simultaneously.",
            target = style("target").yellow(),
            target_path = style("target_path").yellow()
        ).into()),
    };

    let path = resolve_package_path(name)?;
    let values_directory_path = path.join("variables");

    let variables = load_variables(&values_directory_path)?;
    let variables = evaluate(&variables)?;

    let mut builder = RendererBuilder::new();

    for (key, value) in variables.as_mapping().unwrap() {
        builder = builder.with_value(key.as_str().unwrap().to_string(), value);
    }

    let mut args_value: Value = Value::Mapping(Mapping::new());

    for arg in &spell.args {
        let value = matches.get_one::<String>(&arg.name);
        match value {
            Some(v) => {
                let mapping = args_value.as_mapping_mut().unwrap();
                let name = arg.name.to_case(Case::Snake);
                mapping.insert(name.into(), v.clone().into());
            },
            None => {
                if let Some(required) = arg.required {
                    if required {
                        return Err(format!("Invalid usage. Option {} is required", style(&arg.name).yellow()).into())
                    }
                }
            }
        }
    }

    // if let Some(path) = &overrides_path {
    //     if path.is_directory() {
    //         load_variables(path)
    //     }
    // }

    builder = builder.with_value("args", &args_value);

    let templates_directory_path = path.join("templates");
    let templates = load_templates(templates_directory_path);

    for (key, value) in &templates {
        builder = builder.with_file_template(key, value);
    }

    let tera = &builder.build_tera()?;
    let lua = &builder.build_lua()?;
    let noop = &builder.build_noop()?;

    for (key, _value) in &templates {
        let pb = create_progress_bar();
        pb.set_message(format!("Rendering {} template...", style(key).yellow()));
        let file_path = lua.eval_string_template(format!("`{}`", key))?;
        let file_relative_path = RelativePathBuf::from(&file_path);

        let (target_file_relative_path, file_content) = match &file_relative_path.extension() {
            Some("tera") => (
                file_relative_path.with_extension("".to_string()),
                tera.render(key)?
            ),
            Some("luau") => (
                file_relative_path.with_extension("".to_string()),
                lua.render(key)?
            ),
            _ => (
                file_relative_path.clone(),
                noop.render(key)?
            )
        };

        match dry_run {
            true => {
                pb.finish_with_message(format!("{} Rendered template: {}", style("[DONE]").green(), &file_path));
            }
            false => {
                let file_path = target_file_relative_path.to_path(&resolved_target_path);
                pb.finish_with_message(format!("{} Generated file: {}", style("[DONE]").green(), file_path.to_str().unwrap().to_string()));
                save_target_file(&resolved_target_path, &target_file_relative_path, &file_content)?;
            }
        }
    }

    Ok(())
}
