use std::collections::BTreeMap;
use std::error::Error;

use crate::config::{ActionEntry, PackageConfig};
use crate::ResolvedContext;

pub struct ActionRef {
    pub package: PackageConfig,
    pub action: ActionEntry,
    pub full_name: String,
}

pub fn list_actions(context: &ResolvedContext) -> BTreeMap<String, ActionRef> {
    let mut actions: BTreeMap<String, ActionRef> = BTreeMap::new();

    for action in &context.current_config.actions {
        let full_name = format!("@/{}", action.name);
        actions.insert(full_name.clone(), ActionRef {
            package: context.current_config.clone(),
            action: action.clone(),
            full_name: full_name.to_string(),
        });
    }

    actions
}

pub fn find_action_by_name(_context: &ResolvedContext, package: &PackageConfig, name: &String) -> Option<ActionEntry> {
    package.actions
        .iter()
        .find(|e| name.eq(&e.name))
        .map(|e| e.clone())
}

pub fn find_action_by_full_name(context: &ResolvedContext, name: &String) -> Result<ActionRef, Box<dyn Error>> {
    let package = &context.current_config;
    match package.actions.iter().find(|a| name.eq(&format!("@/{}", a.name))) {
        None => Err(format!("No action with name: {}", name).into()),
        Some(action) => Ok(ActionRef {
            package: package.clone(),
            action: action.clone(),
            full_name: format!("@/{}", action.name),
        })
    }
}
