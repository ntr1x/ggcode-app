use std::error::Error;

use clap::{ArgMatches, Command, command};

use ggcode_core::Context;

use crate::commands::action::{create_action_command, execute_action_command};
use crate::commands::completions::{create_autocomplete_command, execute_autocomplete_command};
use crate::commands::generate::{create_generate_command, execute_generate_command};
use crate::commands::init::{create_init_command, execute_init_command};
use crate::commands::install::{create_install_command, execute_install_command};
use crate::commands::repository::{create_repository_command, execute_repository_command};
use crate::commands::run::{create_run_command, execute_run_command};
use crate::commands::scroll::{create_scroll_command, execute_scroll_command};
use crate::commands::target::{create_target_command, execute_target_command};

mod init;
mod install;
mod repository;
mod target;
mod scroll;
mod completions;
mod generate;
mod action;
mod run;

pub fn create_cli_command(context: &Context) -> Result<Command, Box<dyn Error>> {
    let command = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand_required(false)
        .subcommand(create_init_command())
        .subcommand(create_install_command())
        .subcommand(create_generate_command(context)?)
        .subcommand(create_run_command(context)?)
        // .next_help_heading("Manage")
        .subcommand(create_repository_command())
        .subcommand(create_scroll_command())
        .subcommand(create_action_command())
        .subcommand(create_target_command())
        // .subcommand(create_history_command())
        // .next_help_heading("Miscellaneous")
        .subcommand(create_autocomplete_command());

    Ok(command)
}

pub fn execute_cli_command(context: &Context, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("init", sub_matches)) => execute_init_command(&context, sub_matches),
        Some(("install", sub_matches)) => execute_install_command(context.resolve()?, sub_matches),
        Some(("generate", sub_matches)) => execute_generate_command(&context.resolve()?, sub_matches),
        Some(("run", sub_matches)) => execute_run_command(&context.resolve()?, sub_matches),
        Some(("repository", sub_matches)) => execute_repository_command(context.resolve()?, sub_matches),
        Some(("target", sub_matches)) => execute_target_command(context.resolve()?, sub_matches),
        Some(("scroll", sub_matches)) => execute_scroll_command(&context.resolve()?, sub_matches),
        Some(("action", sub_matches)) => execute_action_command(&context.resolve()?, sub_matches),
        Some(("completions", sub_matches)) => execute_autocomplete_command(&context, sub_matches),
        _ => return Err("Unsupported command".into())
    }
}
