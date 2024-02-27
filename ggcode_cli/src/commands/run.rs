use std::env;
use std::error::Error;

use clap::{Arg, ArgMatches, Command};
use console::style;
use convert_case::{Case, Casing};
use relative_path::RelativePathBuf;
use serde_yaml::{Mapping, Value};

use ggcode_core::{Context, ResolvedContext};
use ggcode_core::action::{ActionRef, find_action_by_full_name, list_actions};
use ggcode_core::generator::DefaultGenerator;
use ggcode_core::renderer::luau_evaluator::LuauEvaluatorBuilder;
use ggcode_core::renderer::luau_extras::LuauEngine;
use ggcode_core::storage::{load_string, resolve_search_locations};

pub fn create_run_command(context: &Context) -> Result<Command, Box<dyn Error>> {
    let mut command = Command::new("run")
        .about("Run command")
        .alias("r")
        .arg_required_else_help(true);

    if let Some(resolved_context) = context.resolve().ok() {
        let actions = list_actions(&resolved_context);
        for (name, action) in actions {
            let subcommand = create_run_action_command(&resolved_context, &name, &action)?;
            command = command.subcommand(subcommand);
        }
    }

    Ok(command)
}

pub fn create_run_action_command(
    _context: &ResolvedContext,
    action_name: &String,
    action: &ActionRef
) -> Result<Command, Box<dyn Error>> {
    let mut command = Command::new(action_name.clone())
        .about(&action.action.about.clone().unwrap_or(format!("Run {} action", &action.action.name)));

    for arg_entry in &action.action.args {
        let arg = Arg::new(&arg_entry.name)
            .long(&arg_entry.name)
            .help(arg_entry.about.clone().unwrap_or("Magical spell option".to_string()))
            .required(arg_entry.required.unwrap_or(false));
        command = command.arg(arg);
    }

    Ok(command)
}

pub fn execute_run_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some((path, sub_matches)) => execute_run_action_command(context, &path.to_string(), sub_matches),
        _ => unreachable!()
    }
}

pub fn execute_run_action_command(context: &ResolvedContext, name: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();

    let action = find_action_by_full_name(context, name)?;

    let mut builder = LuauEvaluatorBuilder::new();

    let mut args_value: Value = Value::Mapping(Mapping::new());

    for arg in &action.action.args {
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

    let search_locations = resolve_search_locations(&context.current_config);

    for rp in search_locations {
        builder = builder.with_path_entry(&rp.to_path(&current_dir));
    }

    builder = builder.with_global("args", &args_value);

    let generator = DefaultGenerator {
        context: context.clone(),
        wrapped_observers: vec![]
    };

    builder = builder.enable_engine(LuauEngine {
        context: context.clone(),
        generator
    });

    let evaluator = builder.build()?;

    let action = find_action_by_full_name(&context, name)?;
    let script = load_string(&action.package, &RelativePathBuf::from(action.action.path))?;

    evaluator.eval_value(&script)?;

    Ok(())
}
