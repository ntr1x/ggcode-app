use std::error::Error;

use clap::{Arg, arg, ArgMatches, Command};

use ggcode_core::{Context, ResolvedContext};
use ggcode_core::generator::DefaultGenerator;
use ggcode_core::scroll::{list_scrolls, ScrollRef};
use ggcode_core::storage::resolve_target;

pub fn create_generate_command(context: &Context) -> Result<Command, Box<dyn Error>> {
    let mut command = Command::new("generate")
        .about("Execute generation script from scroll")
        .alias("g")
        .arg_required_else_help(true);

    if let Some(resolved_context) = context.resolve().ok() {
        let scrolls = list_scrolls(&resolved_context);
        for (_, scroll) in scrolls {
            let subcommand = create_generate_scroll_command(&resolved_context, &scroll)?;
            command = command.subcommand(subcommand);
        }
    }

    Ok(command)
}

pub fn create_generate_scroll_command(
    _context: &ResolvedContext,
    scroll: &ScrollRef
) -> Result<Command, Box<dyn Error>> {
    let command = Command::new(&scroll.full_name.clone())
        .about(&scroll.scroll.about.clone().unwrap_or("Casting a magical spell".to_string()))
        .arg(arg!(-t --target <target> "The name of a well-known target")
            .help_heading("Target Options")
            .required_unless_present("target-path"))
        .arg(Arg::new("target-path")
            .long("target-path")
            .short('p')
            .help("The path to output directory")
            .help_heading("Target")
            .required_unless_present("target"))
        .arg(arg!(-v --variables <path> "Path to a file or directory containing variable overrides")
            .required(false))
        .arg(Arg::new("dry-run")
            .long("dry-run".to_string())
            .short('d')
            .num_args(0)
            .help("Do not generate files; simply test the ability to render templates"))
        .arg_required_else_help(true);

    Ok(command)
}

pub fn execute_generate_command(context: &ResolvedContext, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some((name, sub_matches)) => execute_generate_scroll_command(context, &name.to_string(), sub_matches),
        _ => unreachable!()
    }
}

pub fn execute_generate_scroll_command(context: &ResolvedContext, name: &String, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let target_name = matches.get_one::<String>("target");
    let target_path = matches.get_one::<String>("target-path");

    let dry_run = matches.get_one::<bool>("dry-run").unwrap();

    let resolved_target_path = resolve_target(
        &context,
        target_name.map(|d| d.clone()),
        target_path.map(|d| d.clone()))?;

    let generator = DefaultGenerator {
        context: context.clone(),
        wrapped_observers: vec![]
    };

    generator.generate(name, &resolved_target_path, *dry_run, None)?;

    Ok(())
}
