use std::env;

use ggcode_core::config::DEFAULT_CONFIG_NAME;
use ggcode_core::Context;

use crate::commands::{create_cli_command, execute_cli_command};
use crate::config::{load_config, resolve_inner_path};

mod config;
mod commands;
mod structure;

pub fn load_context() -> Result<Context, Box<dyn std::error::Error>> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = load_context()?;

    let cli = create_cli_command(&context);
    let matches = cli.get_matches();
    execute_cli_command(&context, &matches)
}
