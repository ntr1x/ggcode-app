use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ResolvedContext;
use crate::storage::{load_chain, load_config, resolve_inner_path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub commands: Vec<ChainCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCommand {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<ChainArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainArg {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub required: Option<bool>,
}

pub struct ChainRef {
    pub chain: Option<ChainConfig>,
    pub chain_path: String,
}

pub fn list_chains(context: &ResolvedContext) -> BTreeMap<String, ChainRef> {
    let mut chains: BTreeMap<String, ChainRef> = BTreeMap::new();

    for repository in context.current_config.repositories.iter() {
        let repository_path = format!("ggcode_modules/{}", repository.name);
        let config_path = format!("{}/ggcode-info.yaml", repository_path);
        let config = resolve_inner_path(&config_path)
            .ok()
            .and_then(|path| load_config(&path).ok());

        match config {
            None => {},
            Some(repository_config) => {
                for chain_entry in repository_config.chains {
                    let chain_config_path = format!("{}/{}/ggcode-chain.yaml", repository_path, chain_entry.path);
                    let chain = resolve_inner_path(&chain_config_path)
                        .ok()
                        .and_then(|path| load_chain(&path).ok());

                    let key = format!("{}/{}", repository.name, chain_entry.path);
                    chains.insert(key, ChainRef {
                        chain,
                        chain_path: chain_entry.path,
                    });
                }
            },
        }
    }

    for chain_entry in &context.current_config.chains {
        let chain_config_path = format!("{}/ggcode-chain.yaml", chain_entry.path);
        let chain = resolve_inner_path(&chain_config_path)
            .ok()
            .and_then(|path| load_chain(&path).ok());
        let key = format!("@/{}", chain_entry.path);
        chains.insert(key, ChainRef {
            chain,
            chain_path: chain_entry.path.clone(),
        });
    }

    chains
}
