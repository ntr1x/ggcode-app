use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use serde_yaml::{to_value, Value};

#[derive(Default)]
pub struct ConfigBuilder {
    pub(crate) raw_configs: BTreeMap<String, String>,
    pub(crate) file_configs: BTreeMap<String, PathBuf>,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn with_raw_config<S: Into<String>, V: Into<String>>(mut self, name: S, raw: V) -> ConfigBuilder {
        self.raw_configs.insert(name.into(), raw.into());
        self
    }

    pub fn with_file_config<S: Into<String>, V: Into<PathBuf>>(mut self, name: S, path: V) -> ConfigBuilder {
        self.file_configs.insert(name.into(), path.into());
        self
    }
}
