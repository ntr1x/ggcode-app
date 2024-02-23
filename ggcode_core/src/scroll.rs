use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ResolvedContext;
use crate::storage::{load_config, load_scroll, resolve_inner_path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollConfig {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub commands: Vec<ScrollCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollCommand {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<ScrollArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollArg {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub required: Option<bool>,
}

pub struct ScrollRef {
    pub scroll: Option<ScrollConfig>,
    pub scroll_path: String,
}

pub fn list_scrolls(context: &ResolvedContext) -> BTreeMap<String, ScrollRef> {
    let mut scrolls: BTreeMap<String, ScrollRef> = BTreeMap::new();

    for repository in context.current_config.repositories.iter() {
        let repository_path = format!("ggcode_modules/{}", repository.name);
        let config_path = format!("{}/ggcode-info.yaml", repository_path);
        let config = resolve_inner_path(&config_path)
            .ok()
            .and_then(|path| load_config(&path).ok());

        match config {
            None => {},
            Some(repository_config) => {
                for scroll_entry in repository_config.scrolls {
                    let scroll_config_path = format!("{}/{}/ggcode-scroll.yaml", repository_path, scroll_entry.path);
                    let scroll = resolve_inner_path(&scroll_config_path)
                        .ok()
                        .and_then(|path| load_scroll(&path).ok());

                    let key = format!("{}/{}", repository.name, scroll_entry.path);
                    scrolls.insert(key, ScrollRef {
                        scroll,
                        scroll_path: scroll_entry.path,
                    });
                }
            },
        }
    }

    for scroll_entry in &context.current_config.scrolls {
        let scroll_config_path = format!("{}/ggcode-scroll.yaml", scroll_entry.path);
        let scroll = resolve_inner_path(&scroll_config_path)
            .ok()
            .and_then(|path| load_scroll(&path).ok());
        let key = format!("@/{}", scroll_entry.path);
        scrolls.insert(key, ScrollRef {
            scroll,
            scroll_path: scroll_entry.path.clone(),
        });
    }

    scrolls
}