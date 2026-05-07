//! Generic host UI snapshot + batched patches (extend [`SurfacePatch`] for editors, settings, etc.).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::modules::ModulesConfig;
use crate::config::ConfigFile;
use crate::navigation;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "surface";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSnapshot {
    pub modules: BTreeMap<String, bool>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SurfacePatch {
    ModulesSet { name: String, enabled: bool },
}

fn snapshot(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return "{}".to_string();
    };
    let navigation_registry: serde_json::Value =
        serde_json::from_str(&navigation::default_navigation_registry_json())
            .unwrap_or_else(|_| serde_json::json!({}));
    let snap = SurfaceSnapshot {
        modules: cfg.modules.clone(),
        extra: serde_json::json!({
            "navigation_registry": navigation_registry,
        }),
    };
    serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string())
}

fn patch(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(payload) = args.first().copied() else {
        return "Usage: surface.patch '<json-array-of-patch-objects>'".to_string();
    };
    let patches: Vec<SurfacePatch> = match serde_json::from_str(payload) {
        Ok(p) => p,
        Err(e) => return format!("Invalid surface.patch JSON: {e}"),
    };
    let mut messages = Vec::new();
    for p in patches {
        match p {
            SurfacePatch::ModulesSet { name, enabled } => {
                let mut cfg = match ModulesConfig::load_or_create() {
                    Ok(c) => c,
                    Err(e) => return format!("Error loading config: {e}"),
                };
                if let Err(e) = cfg.set_module_state(&name, enabled) {
                    return e;
                }
                if let Err(e) = cfg.save() {
                    return format!("Error saving config: {e}");
                }
                messages.push(format!(
                    "Module {name} {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
            }
        }
    }
    if messages.is_empty() {
        "No patches applied".to_string()
    } else {
        messages.join("\n")
    }
}

/// Helpers for surfaces that don't depend on `serde_json` directly.
pub fn snapshot_module_rows(payload: &str) -> Vec<(String, bool)> {
    serde_json::from_str::<SurfaceSnapshot>(payload)
        .map(|s| s.modules.into_iter().collect())
        .unwrap_or_default()
}

pub fn patch_json_modules_set(name: &str, enabled: bool) -> String {
    serde_json::json!([{
        "op": "modules_set",
        "name": name,
        "enabled": enabled,
    }])
    .to_string()
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "snapshot",
            description: "JSON SurfaceSnapshot (modules + extra.navigation_registry + future buckets)",
            run: snapshot,
        },
        ModuleCommand {
            name: "patch",
            description: "Apply SurfacePatch JSON array (e.g. modules_set); extend enum for editor/settings ops",
            run: patch,
        },
    ]
}
