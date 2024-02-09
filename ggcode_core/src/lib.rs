use std::error::Error;
use std::path::PathBuf;

pub mod config;

use config::Config;
pub mod scroll;

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
            None => {
                return Err("Cannot resolve context".into())
            }
        }
    }
}
