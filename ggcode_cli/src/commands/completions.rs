use std::error::Error;

use clap::{arg, ArgAction, ArgMatches, Command, value_parser};
use clap_complete::{generate, Generator, Shell};

use ggcode_core::Context;

use crate::commands::create_cli_command;

pub fn create_autocomplete_command() -> Command {
    Command::new("completions")
        .about("Generates completions script")
        .arg_required_else_help(true)
        .arg(arg!(-s --shell).action(ArgAction::Set).value_parser(value_parser!(Shell)))
}

pub fn execute_autocomplete_command(context: &Context, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let generator = matches.get_one::<Shell>("shell").copied().unwrap();

    let mut cmd = create_cli_command(context)?;
    eprintln!("Generating completion file for {generator}...");
    print_completions(generator, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}