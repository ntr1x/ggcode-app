use std::collections::BTreeMap;
use std::error::Error;

use crate::config::{PackageConfig, ScrollEntry};
use crate::ResolvedContext;
use crate::storage::{load_config, resolve_inner_path};

#[derive(Debug, Clone)]
pub struct ScrollRef {
    pub package: PackageConfig,
    pub scroll: ScrollEntry,
    pub full_name: String,
    pub dependency_name: Option<String>,
}

pub fn list_scrolls(context: &ResolvedContext) -> BTreeMap<String, ScrollRef> {
    let mut scrolls: BTreeMap<String, ScrollRef> = BTreeMap::new();

    for repository in context.current_config.repositories.iter() {
        let repository_path = format!("ggcode_modules/{}", repository.name);
        let config_path = format!("{}/ggcode-info.yaml", repository_path);
        let config = resolve_inner_path(&config_path)
            .ok()
            .and_then(|path| load_config(&path).ok());

        match &config {
            None => {},
            Some(repository_config) => {
                for scroll_entry in &repository_config.scrolls {
                    let full_name = format!("{}/{}", repository.name, scroll_entry.name);
                    scrolls.insert(full_name.clone(), ScrollRef {
                        package: repository_config.clone(),
                        scroll: scroll_entry.clone(),
                        full_name,
                        dependency_name: Some(repository.name.clone()),
                    });
                }
            },
        }
    }

    for scroll_entry in &context.current_config.scrolls {
        let full_name = format!("@/{}", scroll_entry.name);
        scrolls.insert(full_name.clone(), ScrollRef {
            package: context.current_config.clone(),
            scroll: scroll_entry.clone(),
            full_name,
            dependency_name: None,
        });
    }

    scrolls
}

pub fn find_scroll_by_name(_context: &ResolvedContext, package: &PackageConfig, name: &String) -> Option<ScrollEntry> {
    package.scrolls
        .iter()
        .find(|e| name.eq(&e.name))
        .map(|e| e.clone())
}

pub fn find_scroll_by_full_name(context: &ResolvedContext, name: &String) -> Result<ScrollRef, Box<dyn Error>> {
    let package = &context.current_config;

    for scroll in &package.scrolls {
        let full_name = format!("@/{}", scroll.name);
        if *name == full_name {
            return Ok(ScrollRef {
                package: package.clone(),
                scroll: scroll.clone(),
                full_name: format!("@/{}", scroll.name),
                dependency_name: None
            })
        }
    }

    for repository in &context.current_config.repositories {
        let repository_path = format!("ggcode_modules/{}", repository.name);
        let config_path = format!("{}/ggcode-info.yaml", repository_path);
        let config = resolve_inner_path(&config_path)
            .ok()
            .and_then(|path| load_config(&path).ok());

        match config {
            None => {},
            Some(repository_config) => {
                for scroll in repository_config.scrolls {
                    let full_name = format!("{}/{}", repository.name, scroll.name);
                    if *name == full_name {
                        return Ok(ScrollRef {
                            package: context.current_config.clone(),
                            scroll: scroll.clone(),
                            full_name,
                            dependency_name: Some(repository.name.clone()),
                        })
                    }
                }
            },
        }
    }

    Err(format!("No scroll with name: {}", name).into())
}
