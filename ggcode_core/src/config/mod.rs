use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_NAME: &str = "ggcode-info.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub scrolls: Vec<ScrollEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub repositories: Vec<Repository>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub targets: Vec<Target>,
}
