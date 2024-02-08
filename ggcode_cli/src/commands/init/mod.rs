use clap::{ArgMatches, Command};
use dialoguer::{Confirm, Input};
use dialoguer::theme::ColorfulTheme;

use ggcode_core::Context;
use ggcode_core::config::{Config, RepositoryEntry, ScrollEntry, TargetEntry};
use crate::config::{resolve_inner_path, save_config};

const REPOSITORY_CENTRAL_NAME: &str = "central";
const REPOSITORY_CENTRAL_URI: &str = "git@github.com:ntr1x/com.ntr1x.setup.git";

const TARGET_WORKDIR_NAME: &str = "@";
const TARGET_WORKDIR_PATH: &str = ".";

pub fn create_init_command() -> Command {
    Command::new("init")
        .about("Creates a ggcode-info.yaml file")
        // .allow_external_subcommands(true)
}

pub fn execute_init_command(context: &Context, _: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .default(context.current_config
            .as_ref()
            .map(|config| -> String { format!("{}", config.name) })
            .unwrap_or(context.directory_name.clone())
        )
        .interact_text()
        .unwrap();

    let repositories = setup_repositories(&context.current_config);
    let targets = setup_targets(&context.current_config);
    let scrolls = setup_scrolls(&context.current_config);

    let config = Config {
        name,
        scrolls,
        targets,
        repositories,
    };

    save_config(&resolve_inner_path(&context.config_path)?, config)
}

fn setup_scrolls(config: &Option<Config>) -> Vec<ScrollEntry> {
    match config {
        None => vec!(),
        Some(config) => config.scrolls.clone()
    }
}

fn setup_repositories(config: &Option<Config>) -> Vec<RepositoryEntry> {
    let mut repositories: Vec<RepositoryEntry> = vec![];

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
            .with_prompt("Should we add central repository?")
            .default(true)
            .interact()
            .unwrap();

        if add_repository {
            repositories.push(RepositoryEntry {
                name: REPOSITORY_CENTRAL_NAME.to_string(),
                uri: REPOSITORY_CENTRAL_URI.to_string(),
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

    repositories
}

fn setup_targets(config: &Option<Config>) -> Vec<TargetEntry> {
    let mut targets: Vec<TargetEntry> = vec![];

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

    targets
}
