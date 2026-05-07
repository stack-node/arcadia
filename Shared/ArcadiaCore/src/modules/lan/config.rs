use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const NODE_CONFIG_FILE_NAME: &str = "lan_nodes.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanNodeConfig {
    pub auto: bool,
    pub approved_nodes: Vec<String>,
    pub node_rules: BTreeMap<String, bool>,
    pub aliases: BTreeMap<String, String>,
}

impl Default for LanNodeConfig {
    fn default() -> Self {
        Self {
            auto: false,
            approved_nodes: Vec::new(),
            node_rules: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }
}

impl ConfigFile for LanNodeConfig {
    fn file_name() -> &'static str {
        NODE_CONFIG_FILE_NAME
    }
}

pub fn load_node_config() -> Result<LanNodeConfig, String> {
    LanNodeConfig::load_or_create().map_err(|err| err.to_string())
}

pub fn save_node_config(config: &LanNodeConfig) -> Result<(), String> {
    config.save().map_err(|err| err.to_string())
}

pub fn normalize_node_identifier(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn resolve_alias_target(target: &str, cfg: &LanNodeConfig) -> String {
    let key = normalize_node_identifier(target);
    cfg.aliases.get(&key).cloned().unwrap_or(key)
}

pub fn aliases_for_identifier(identifier: &str, cfg: &LanNodeConfig) -> Vec<String> {
    let id = normalize_node_identifier(identifier);
    cfg.aliases
        .iter()
        .filter_map(|(alias, mapped)| {
            if normalize_node_identifier(mapped) == id {
                Some(alias.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn is_identifier_approved(cfg: &LanNodeConfig, ip: &str, hostname: &str) -> bool {
    let ip_key = normalize_node_identifier(ip);
    let host_key = normalize_node_identifier(hostname);
    cfg.approved_nodes.iter().any(|node| {
        let key = normalize_node_identifier(node);
        key == ip_key || key == host_key
    })
}

pub fn is_auto_allowed(ip: &str, hostname: &str) -> bool {
    let Ok(cfg) = load_node_config() else {
        return false;
    };
    // Previously approved nodes always auto-accept — trust was already established.
    if is_identifier_approved(&cfg, ip, hostname) {
        return true;
    }
    if !cfg.auto {
        return false;
    }
    let ip_key = normalize_node_identifier(ip);
    let host_key = normalize_node_identifier(hostname);
    if let Some(value) = cfg
        .node_rules
        .get(&ip_key)
        .or_else(|| cfg.node_rules.get(&host_key))
    {
        return *value;
    }
    cfg.approved_nodes.iter().any(|node| {
        let key = normalize_node_identifier(node);
        key == ip_key || key == host_key
    })
}
