use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::ConfigFile;

const LEGACY_LAN_MODULE_NAME: &str = "lan-module";
pub const LAN_MODULE_NAME: &str = "lan";
pub const NET_MODULE_NAME: &str = "net";
pub const SHELL_MODULE_NAME: &str = "shell";
const FILE_NAME: &str = "modules.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub modules: BTreeMap<String, bool>,
}

fn required_modules(module_name: &str) -> &'static [&'static str] {
    match module_name {
        LAN_MODULE_NAME => &[NET_MODULE_NAME],
        _ => &[],
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        let mut modules = BTreeMap::new();
        modules.insert(LAN_MODULE_NAME.to_string(), false);
        modules.insert(NET_MODULE_NAME.to_string(), false);
        modules.insert(SHELL_MODULE_NAME.to_string(), false);
        Self { modules }
    }
}

impl ModulesConfig {
    pub fn required_modules_for(module_name: &str) -> Result<&'static [&'static str], String> {
        match module_name {
            LAN_MODULE_NAME | NET_MODULE_NAME | SHELL_MODULE_NAME => {
                Ok(required_modules(module_name))
            }
            _ => Err("Unknown module key".to_string()),
        }
    }

    pub fn missing_requirements_for(&self, module_name: &str) -> Result<Vec<String>, String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }
        let mut missing = Vec::new();
        self.collect_missing_requirements(module_name, &mut missing)?;
        missing.sort();
        missing.dedup();
        Ok(missing)
    }

    fn collect_missing_requirements(
        &self,
        module_name: &str,
        missing: &mut Vec<String>,
    ) -> Result<(), String> {
        for required in Self::required_modules_for(module_name)? {
            let Some(required_enabled) = self.modules.get(*required) else {
                return Err(format!(
                    "Cannot enable {module_name}: required module {required} is missing"
                ));
            };
            if !required_enabled {
                missing.push((*required).to_string());
            }
            self.collect_missing_requirements(required, missing)?;
        }
        Ok(())
    }

    pub fn enable_with_requirements(&mut self, module_name: &str) -> Result<(), String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }

        for required in required_modules(module_name) {
            if !self.modules.contains_key(*required) {
                return Err(format!(
                    "Cannot enable {module_name}: required module {required} is missing"
                ));
            }
            self.enable_with_requirements(required)?;
        }

        self.set_module_state(module_name, true)
    }

    pub fn set_module_state(&mut self, module_name: &str, enabled: bool) -> Result<(), String> {
        if !self.modules.contains_key(module_name) {
            return Err("Unknown module key".to_string());
        }

        if enabled {
            for required in required_modules(module_name) {
                let Some(required_enabled) = self.modules.get(*required) else {
                    return Err(format!(
                        "Cannot enable {module_name}: required module {required} is missing"
                    ));
                };
                if !required_enabled {
                    return Err(format!(
                        "Cannot enable {module_name}: requires {required} to be enabled"
                    ));
                }
            }
        } else {
            let blocking_dependents = self
                .modules
                .iter()
                .filter(|(_, is_enabled)| **is_enabled)
                .filter_map(|(name, _)| {
                    if required_modules(name).contains(&module_name) {
                        Some(name.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !blocking_dependents.is_empty() {
                return Err(format!(
                    "Cannot disable {module_name}: required by enabled module(s): {}",
                    blocking_dependents.join(", ")
                ));
            }
        }

        if let Some(entry) = self.modules.get_mut(module_name) {
            *entry = enabled;
        }
        Ok(())
    }
}

impl ConfigFile for ModulesConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }

    fn merge_defaults(&mut self) -> bool {
        let mut changed = false;

        if let Some(legacy_value) = self.modules.remove(LEGACY_LAN_MODULE_NAME) {
            self.modules
                .entry(LAN_MODULE_NAME.to_string())
                .or_insert(legacy_value);
            changed = true;
        }
        if self.modules.remove("lan-mobile").is_some() {
            changed = true;
        }

        let defaults = Self::default();
        for (key, value) in defaults.modules {
            if !self.modules.contains_key(&key) {
                self.modules.insert(key, value);
                changed = true;
            }
        }
        changed
    }
}
