use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use clap::{ArgMatches, Command};
use console::style;
use relative_path::RelativePathBuf;
use ggcode_core::config::RepositoryEntry;

use ggcode_core::ResolvedContext;
use crate::storage::{load_config, resolve_inner_path};
use crate::greetings::create_progress_bar;

pub fn create_install_command() -> Command {
    Command::new("install")
        .about("Recursively fetches dependent repositories")
        .alias("i")
}

pub fn execute_install_command(context: ResolvedContext, _matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let modules_path = resolve_inner_path(&"ggcode_modules".to_string())?;
    let mut repository_map: BTreeMap<String, RepositoryEntry> = BTreeMap::new();
    resolve_repository_deps(&current_dir, &modules_path, &context.current_config.repositories, &mut repository_map);

    Ok(())
}

fn resolve_repository_deps(current_dir: &PathBuf, modules_path: &RelativePathBuf, repositories: &Vec<RepositoryEntry>, repository_map: &mut BTreeMap<String, RepositoryEntry>) {
    for repository in repositories {
        match repository_map.get(&repository.name) {
            None => {
                download_repo(&current_dir, &modules_path, &repository.name, repository);
                repository_map.insert(repository.name.clone(), repository.clone());
                let repository_relative_path = modules_path.join(format!("{}/ggcode-info.yaml", repository.name));
                match load_config(&repository_relative_path).ok() {
                    None => {},
                    Some(config) => {
                        resolve_repository_deps(&current_dir, &modules_path, &config.repositories, repository_map);
                    }
                }
            },
            Some(_) => {}
        }
    }
}

fn download_repo(current_dir: &PathBuf, modules_path: &RelativePathBuf, name: &String, repository: &RepositoryEntry) {
    let target_relative_path = modules_path.join(name);
    let target_path = target_relative_path.to_path(&current_dir);
    let target_path_string = target_relative_path.into_string();

    let pb = create_progress_bar();
    pb.set_message(format!("Updating `{}` repository", repository.name));

    match target_path.exists() {
        false => {
            pb.set_message(format!(
                "Directory `{}` does not exist. Cloning `{}` repo into this directory.",
                &target_path_string,
                repository.uri));

            let o = std::process::Command::new("git")
                .args([
                    "clone",
                    &repository.uri,
                    &target_path.into_os_string().into_string().unwrap().to_string()
                ])
                .output()
                .is_ok_and(|out| out.status.success());

            match o {
                false => {
                    pb.finish_with_message(format!(
                        "{} Cannot clone `{}` repo into the `{}` directory.",
                        style("[FAIL]").red(),
                        repository.uri,
                        &target_path_string));
                },
                true => {
                    pb.finish_with_message(format!(
                        "{} Repository `{}` cloned into the `{}` directory.",
                        style("[DONE]").green(),
                        repository.uri,
                        &target_path_string));
                }
            }
        },
        true => {
            pb.set_message(format!(
                "Directory {} exists. Resetting from {} repo.",
                &target_path_string,
                repository.uri));

            let o1 = std::process::Command::new("git")
                .current_dir(&target_path)
                .args(["fetch", "origin"])
                .output()
                .is_ok_and(|out| out.status.success());

            match o1 {
                false => {
                    pb.finish_with_message(format!(
                        "{} Cannot fetch changes from `{}` repo into the `{}` directory.",
                        style("[FAIL]").red(),
                        repository.uri,
                        &target_path_string));
                },
                true => {
                    let o2 = std::process::Command::new("git")
                        .current_dir(&target_path)
                        .args(["reset", "--hard", "@{upstream}"])
                        .output()
                        .is_ok_and(|out| out.status.success());

                    match o2 {
                        false => {
                            pb.finish_with_message(format!(
                                "{} Cannot reset `{}` repo in the `{}` directory.",
                                style("[FAIL]").red(),
                                repository.uri,
                                &target_path_string));
                        },
                        true => {
                            pb.finish_with_message(format!(
                                "{} Repository `{}` updated",
                                style("[DONE]").green(),
                                repository.name));
                        }
                    }
                }
            }
        }
    };
}
