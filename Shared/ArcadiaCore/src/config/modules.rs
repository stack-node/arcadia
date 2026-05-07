use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::ConfigFile;

const LEGACY_LAN_MODULE_NAME: &str = "lan-module";
pub const LAN_MODULE_NAME: &str = "lan";
pub const NET_MODULE_NAME: &str = "net";
pub const SURFACE_MODULE_NAME: &str = "surface";
pub const REMOTE_SESSION_MODULE_NAME: &str = "remote-session";
pub const SHELL_MODULE_NAME: &str = "shell";
pub const SHELL_MOTD_MODULE_NAME: &str = "shell-motd";
const FILE_NAME: &str = "modules.toml";

#[derive(Debug, Clone, Copy)]
pub struct ModuleManifest {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str],
}

// Single source of truth for modules and their metadata.
static MODULE_REGISTRY: &[ModuleManifest] = &[
    ModuleManifest {
        name: LAN_MODULE_NAME,
        version: "1.0.0",
        description: "Local network discovery and peer communication.",
        required_modules: &[NET_MODULE_NAME],
    },
    ModuleManifest {
        name: NET_MODULE_NAME,
        version: "1.0.0",
        description: "Shared networking foundation for routed module commands.",
        required_modules: &[],
    },
    ModuleManifest {
        name: SURFACE_MODULE_NAME,
        version: "0.1.0",
        description: "Generic UI snapshot (surface.snapshot) and patches (surface.patch); extend patches for new surfaces.",
        required_modules: &[],
    },
    ModuleManifest {
        name: REMOTE_SESSION_MODULE_NAME,
        version: "0.1.0",
        description: "Permission to route execute_command over LAN (net_as: lan:…); transcript/mirror are automatic on hosts.",
        required_modules: &[NET_MODULE_NAME, LAN_MODULE_NAME],
    },
    ModuleManifest {
        name: SHELL_MODULE_NAME,
        version: "1.0.0",
        description: "Interactive shell command execution for Arcadia surfaces.",
        required_modules: &[],
    },
    ModuleManifest {
        name: SHELL_MOTD_MODULE_NAME,
        version: "1.0.0",
        description: "Fastfetch-style banner when opening the Arcadia shell (requires shell).",
        required_modules: &[SHELL_MODULE_NAME],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub modules: BTreeMap<String, bool>,
}

fn required_modules(module_name: &str) -> &'static [&'static str] {
    MODULE_REGISTRY
        .iter()
        .find(|manifest| manifest.name == module_name)
        .map(|manifest| manifest.required_modules)
        .unwrap_or(&[])
}

fn is_known_module(module_name: &str) -> bool {
    MODULE_REGISTRY
        .iter()
        .any(|manifest| manifest.name == module_name)
}

impl Default for ModulesConfig {
    fn default() -> Self {
        let modules = MODULE_REGISTRY
            .iter()
            .map(|manifest| {
                let enabled = manifest.name == SURFACE_MODULE_NAME;
                (manifest.name.to_string(), enabled)
            })
            .collect();
        Self { modules }
    }
}

impl ModulesConfig {
    pub fn manifest_for(module_name: &str) -> Option<&'static ModuleManifest> {
        MODULE_REGISTRY
            .iter()
            .find(|manifest| manifest.name == module_name)
    }

    pub fn required_modules_for(module_name: &str) -> Result<&'static [&'static str], String> {
        if is_known_module(module_name) {
            Ok(required_modules(module_name))
        } else {
            Err("Unknown module key".to_string())
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
