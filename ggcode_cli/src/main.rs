use std::env;
use std::error::Error;
use std::process::ExitCode;

use ::console::style;

use ggcode_core::config::DEFAULT_CONFIG_NAME;
use ggcode_core::Context;
use ggcode_core::storage::{load_config, resolve_inner_path};

use crate::commands::{create_cli_command, execute_cli_command};
use crate::greetings::generate_wishes;

mod commands;
mod greetings;
pub mod terminal;

pub fn load_context() -> Result<Context, Box<dyn Error>> {
    let directory_path = env::current_dir()?;
    let directory_name = directory_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let current_config = load_config(&resolve_inner_path(&DEFAULT_CONFIG_NAME.to_string())?).ok();

    let context = Context {
        config_path: DEFAULT_CONFIG_NAME.to_string(),
        directory_path,
        directory_name,
        current_config,
    };

    Ok(context)
}

fn execute() -> Result<(), Box<dyn Error>> {
    let context = load_context()?;
    let cli = create_cli_command(&context)?;
    let matches = cli.get_matches();
    execute_cli_command(&context, &matches)
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => {
            eprintln!("{} {}", style("[SUCCESS]").green(), generate_wishes());
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("{} {}", style("[FAILURE]").red(), e);
            ExitCode::FAILURE
        }
    }
}
