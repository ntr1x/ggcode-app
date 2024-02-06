use std::env;
use std::path::Path;

use clap::{ArgMatches, Command};
use git2::{Cred, RemoteCallbacks};

use ggcode_core::ResolvedContext;

pub fn create_install_command() -> Command {
    Command::new("install")
        .about("Recursively fetches dependent repositories")
        .alias("i")
}

pub fn execute_install_command(context: ResolvedContext, _matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    for repository in context.current_config.repositories {

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
                None,
            )
        });

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        let directory = format!("ggcode_modules/{}" , repository.name);
        /*let repo = */match builder.clone(repository.uri.as_str(), Path::new(directory.as_str())) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
    }

    Ok(())
}
