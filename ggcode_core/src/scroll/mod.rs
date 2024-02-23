use serde::{Deserialize, Serialize};

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
