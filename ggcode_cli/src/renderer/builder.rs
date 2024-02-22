use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use serde_yaml::{to_value, Value};

#[derive(Default)]
pub struct RendererBuilder {
    pub(crate) values: BTreeMap<String, Value>,
    // raw_scripts: BTreeMap<String, String>,
    pub(crate) raw_templates: BTreeMap<String, String>,
    // file_scripts: BTreeMap<String, String>,
    pub(crate) file_templates: BTreeMap<String, PathBuf>,
}

impl RendererBuilder {
    pub fn new() -> RendererBuilder {
        RendererBuilder::default()
    }

    pub fn with_value<S: Into<String>, V: Serialize + ?Sized>(mut self, key: S, value: &V) -> RendererBuilder {
        self.values.insert(key.into(), to_value(value).unwrap());
        self
    }

    #[allow(dead_code)]
    pub fn with_raw_template<S: Into<String>, V: Into<String>>(mut self, name: S, raw: V) -> RendererBuilder {
        self.raw_templates.insert(name.into(), raw.into());
        self
    }

    pub fn with_file_template<S: Into<String>, V: Into<PathBuf>>(mut self, name: S, path: V) -> RendererBuilder {
        self.file_templates.insert(name.into(), path.into());
        self
    }
}
