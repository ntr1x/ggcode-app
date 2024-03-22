use std::error::Error;

use clap::{arg, ArgMatches, Command};
use console::style;
use dialoguer::{Confirm, Input};
use dialoguer::theme::ColorfulTheme;
use lazy_static::lazy_static;
use regex::Regex;

use ggcode_core::config::{ActionEntry, PackageConfig, RepositoryEntry, ScrollEntry, TargetEntry};
use ggcode_core::Context;
use ggcode_core::storage::{resolve_inner_path, save_config};
use crate::terminal::input::TerminalInput;

const REPOSITORY_CORE_NAME: &str = "core";
const REPOSITORY_CORE_URI: &str = "git@github.com:ntr1x/ggcode-repo-core.git";

const TARGET_WORKDIR_NAME: &str = "@";
const TARGET_WORKDIR_PATH: &str = "target";

pub fn create_init_command() -> Command {
    Command::new("init")
        .about("Creates a ggcode-info.yaml file")
        .arg(arg!(-n --name <String> "Project name"))
        .arg(arg!(-r --repository <String> "Add a repository")
            .value_name("Name:URI")
            .help(format!(
                "Add a repository. Usage examples:\n\t{}\n\t{}",
                format!("--repository {}:{}", style("core").yellow(), style("git@github.com:ntr1x/ggcode-repo-core.git").cyan()),
                format!("--repository {}:{}", style("compose").yellow(), style("https://github.com/ntr1x/ggcode-repo-compose.git").cyan()),
            ))
            .num_args(0..))
        .arg(arg!(-t --target <String> "Add a target")
            .value_name("Name:Path")
            .help(format!(
                "Add a target. Usage examples:\n\t{}\n\t{}",
                format!("--target {}:{}", style("@").yellow(), style(".").cyan()),
                format!("--target {}:{}", style("app").yellow(), style("../path/to/my/app").cyan()),
            ))
            .num_args(0..))
        // .arg(arg!(-s --scroll <String> "Add a scroll")
        //     .value_name("Path")
        //     .value_hint(ValueHint::DirPath)
        //     .help(format!(
        //         "Add a scroll. Usage examples:\n\t{}",
        //         format!("--scroll {}", style("scrolls/generate").cyan()),
        //     ))
        //     .num_args(0..))
}

pub fn execute_init_command(context: &Context, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let default_name = context.current_config
        .as_ref()
        .map(|config| -> String { format!("{}", config.name) })
        .unwrap_or(context.directory_name.clone());

    let name = TerminalInput::builder()
        .matches(matches)
        .name("name")
        .prompt("The name of your project")
        .default_value(default_name)
        .required(true)
        .build()?
        .read_string()?
        .unwrap();

    let repositories = setup_repositories(&context.current_config, matches)?;
    let targets = setup_targets(&context.current_config, matches)?;
    let scrolls = setup_scrolls(&context.current_config, matches)?;
    let chains = setup_chains(&context.current_config, matches)?;

    let config = PackageConfig {
        name: name.to_string(),
        scrolls,
        actions: chains,
        targets,
        repositories,
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)
}

fn setup_scrolls(config: &Option<PackageConfig>, _matches: &ArgMatches) -> Result<Vec<ScrollEntry>, Box<dyn Error>> {
    match config {
        None => Ok(vec!()),
        Some(config) => Ok(config.scrolls.clone())
    }
}

fn setup_chains(config: &Option<PackageConfig>, _matches: &ArgMatches) -> Result<Vec<ActionEntry>, Box<dyn Error>> {
    match config {
        None => Ok(vec!()),
        Some(config) => Ok(config.actions.clone())
    }
}

fn setup_repositories(config: &Option<PackageConfig>, matches: &ArgMatches) -> Result<Vec<RepositoryEntry>, Box<dyn Error>> {
    let repository_inputs = matches.get_many::<String>("repository");

    lazy_static! {
        static ref RE_REPOSITORY: Regex = Regex::new(r"^(?P<name>[^:]*):(?P<uri>.*)$").unwrap();
    }

    let mut repositories: Vec<RepositoryEntry> = vec![];
    let mut ask_for_more = true;
    if let Some(values) = repository_inputs {
        ask_for_more = false;
        for value in values {
            match &RE_REPOSITORY.captures(value) {
                None => {
                    return Err(format!("Invalid repository param: {}", value).into())
                },
                Some(v) => {
                    repositories.push(RepositoryEntry {
                        name: v.name("name").unwrap().as_str().to_string(),
                        uri: v.name("uri").unwrap().as_str().to_string()
                    })
                }
            }
        }
    }


    if ask_for_more {
        match config {
            Some(config) => {
                for repository in &config.repositories {
                    let keep_repository = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Should we pick `{}` repository?", repository.name))
                        .default(true)
                        .interact()
                        .unwrap();
                    if keep_repository {
                        repositories.push(repository.clone());
                    }
                }
            },
            None => {}
        }

        if repositories.len() == 0 {
            let add_repository = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Should we add core repository?")
                .default(true)
                .interact()
                .unwrap();

            if add_repository {
                repositories.push(RepositoryEntry {
                    name: REPOSITORY_CORE_NAME.to_string(),
                    uri: REPOSITORY_CORE_URI.to_string(),
                })
            }
        }

        let mut add_repository = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Should we add more repositories?")
            .default(repositories.len() == 0)
            .interact()
            .unwrap();

        while add_repository {
            let name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Repository name (leave blank to break a loop)")
                .allow_empty(true)
                .interact_text()
                .unwrap();

            let uri: String = match name.is_empty() {
                true => "".to_string(),
                false => Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Repository uri (leave blank to break a loop)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap()
            };

            let added = !name.is_empty() && !uri.is_empty();

            if added {
                repositories.push(RepositoryEntry {
                    name,
                    uri
                });
            }

            add_repository = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Should we add more repositories?")
                .default(added)
                .interact()
                .unwrap();
        }
    }

    Ok(repositories)
}

fn setup_targets(config: &Option<PackageConfig>, matches: &ArgMatches) -> Result<Vec<TargetEntry>, Box<dyn Error>> {
    let target_inputs = matches.get_many::<String>("target");

    lazy_static! {
        static ref RE_TARGET: Regex = Regex::new(r"^(?P<name>[^:]*):(?P<path>.*)$").unwrap();
    }

    let mut targets: Vec<TargetEntry> = vec![];
    let mut ask_for_more = true;
    if let Some(values) = target_inputs {
        ask_for_more = false;
        for value in values {
            match &RE_TARGET.captures(value) {
                None => {
                    return Err(format!("Invalid target param: {}", value).into())
                },
                Some(v) => {
                    targets.push(TargetEntry {
                        name: v.name("name").unwrap().as_str().to_string(),
                        path: v.name("path").unwrap().as_str().to_string(),
                    })
                }
            }
        }
    }

    if ask_for_more {
        match config {
            Some(config) => {
                for target in &config.targets {
                    let keep_target = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Should we pick `{}` target?", target.name))
                        .default(true)
                        .interact()
                        .unwrap();
                    if keep_target {
                        targets.push(target.clone());
                    }
                }
            },
            None => {}
        }

        if targets.len() == 0 {
            let add_target = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Should we register working directory as `@` target?")
                .default(true)
                .interact()
                .unwrap();

            if add_target {
                targets.push(TargetEntry {
                    name: TARGET_WORKDIR_NAME.to_string(),
                    path: TARGET_WORKDIR_PATH.to_string(),
                })
            }
        }

        let mut add_target = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Should we add more targets?")
            .default(targets.len() == 0)
            .interact()
            .unwrap();

        while add_target {
            let name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Target name (leave blank to break a loop)")
                .allow_empty(true)
                .interact_text()
                .unwrap();

            let path: String = match name.is_empty() {
                true => "".to_string(),
                false => Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Target path (leave blank to break a loop)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap()
            };

            let added = !name.is_empty() && !path.is_empty();

            if added {
                targets.push(TargetEntry {
                    name,
                    path
                });
            }

            add_target = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Should we add more targets?")
                .default(added)
                .interact()
                .unwrap();
        }
    }

    Ok(targets)
}
