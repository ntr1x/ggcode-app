use std::error::Error;
use clap::{ArgMatches, Command, command};
use ggcode_core::Context;
use crate::commands::history::create_history_command;
use crate::commands::init::{create_init_command, execute_init_command};
use crate::commands::install::{create_install_command, execute_install_command};
use crate::commands::repository::{create_repository_command, execute_repository_command};
use crate::commands::gen_target::{create_target_command, execute_target_command};

pub mod history;
pub mod init;
pub mod install;
pub mod repository;
pub mod gen_target;

pub fn create_cli_command() -> Command {
    command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(create_init_command())
        .subcommand(create_install_command())
        .subcommand(create_repository_command())
        .subcommand(create_target_command())
        .subcommand(create_history_command())
}

pub fn execute_cli_command(context: Context, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("init", sub_matches)) => execute_init_command(context, sub_matches),
        Some(("install", sub_matches)) => execute_install_command(context.resolve()?, sub_matches),
        Some(("repository", sub_matches)) => execute_repository_command(context.resolve()?, sub_matches),
        Some(("target", sub_matches)) => execute_target_command(context.resolve()?, sub_matches),
        _ => unreachable!()
    }
}
