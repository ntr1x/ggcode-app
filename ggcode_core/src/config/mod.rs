use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_NAME: &str = "ggcode-info.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEntry {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub scrolls: Vec<ScrollEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub repositories: Vec<RepositoryEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub targets: Vec<TargetEntry>,
}

pub struct PackageData {
    pub config: PackageConfig,
    pub dependencies: BTreeMap<String, PackageConfig>,
}
