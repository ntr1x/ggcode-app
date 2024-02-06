use std::error::Error;
use std::path::PathBuf;

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
pub struct Config {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub repositories: Vec<Repository>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub config_path: String,
    pub directory_path: PathBuf,
    pub directory_name: String,
    pub current_config: Option<Config>,
}

#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub config_path: String,
    pub directory_path: PathBuf,
    pub directory_name: String,
    pub current_config: Config,
}

impl Context {
    pub fn resolve(&self) -> Result<ResolvedContext, Box<dyn Error>> {
        match &self.current_config {
            Some(config) => Ok(
                ResolvedContext {
                    config_path: self.config_path.clone(),
                    directory_path: self.directory_path.clone(),
                    directory_name: self.directory_name.clone(),
                    current_config: config.clone()
                }
            ),
            None => panic!("Cannot resolve config")
        }
    }
}
