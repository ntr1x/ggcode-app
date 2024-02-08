use std::collections::BTreeMap;
use ggcode_core::ResolvedContext;
use ggcode_core::scroll::Scroll;
use crate::config::{load_config, load_scroll, resolve_inner_path};

pub fn list_scrolls(context: &ResolvedContext) -> BTreeMap<String, Option<Scroll>> {
    let mut scrolls: BTreeMap<String, Option<Scroll>> = BTreeMap::new();

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

                    scrolls.insert(format!("{}/{}", repository.name, scroll_entry.path), scroll);
                }
            },
        }
    }

    for scroll_entry in &context.current_config.scrolls {
        let scroll_config_path = format!("{}/ggcode-scroll.yaml", scroll_entry.path);
        let scroll = resolve_inner_path(&scroll_config_path)
            .ok()
            .and_then(|path| load_scroll(&path).ok());
        scrolls.insert(format!("@/{}", scroll_entry.path), scroll);
    }

    scrolls
}