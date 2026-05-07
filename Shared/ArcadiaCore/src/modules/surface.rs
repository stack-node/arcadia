//! Generic host UI snapshot + batched patches (extend [`SurfacePatch`] for editors, settings, etc.).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::config::modules::ModulesConfig;
use crate::config::ConfigFile;
use crate::navigation::NavigationRegistryOwned;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "surface";

static SURFACE_REVISION: AtomicU64 = AtomicU64::new(1);

pub(crate) fn bump_surface_revision() {
    SURFACE_REVISION.fetch_add(1, Ordering::SeqCst);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceSnapshot {
    pub modules: BTreeMap<String, bool>,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SurfacePatch {
    ModulesSet {
        #[serde(default)]
        client_id: Option<String>,
        name: String,
        enabled: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ParsedSurfaceSnapshot {
    pub modules: Vec<(String, bool)>,
    pub revision: u64,
    pub navigation_registry: Option<NavigationRegistryOwned>,
}

pub fn parse_surface_snapshot(payload: &str) -> ParsedSurfaceSnapshot {
    serde_json::from_str::<SurfaceSnapshot>(payload)
        .map(|s| ParsedSurfaceSnapshot {
            modules: s.modules.into_iter().collect(),
            revision: s.revision,
            navigation_registry: navigation_registry_from_extra(&s.extra),
        })
        .unwrap_or_default()
}

fn navigation_registry_from_extra(extra: &serde_json::Value) -> Option<NavigationRegistryOwned> {
    extra
        .get("navigation_registry")
        .and_then(|v| serde_json::from_value::<NavigationRegistryOwned>(v.clone()).ok())
}

fn snapshot(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return "{}".to_string();
    };
    let revision = SURFACE_REVISION.load(Ordering::SeqCst);
    let navigation_registry = serde_json::to_value(&NavigationRegistryOwned::from_static_registry())
        .unwrap_or_else(|_| serde_json::json!({}));
    let snap = SurfaceSnapshot {
        modules: cfg.modules.clone(),
        revision,
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
            SurfacePatch::ModulesSet {
                name,
                enabled,
                ..
            } => {
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
        return "No patches applied".to_string();
    }
    bump_surface_revision();
    messages.join("\n")
}

/// Helpers for surfaces that don't depend on `serde_json` directly.
pub fn snapshot_module_rows(payload: &str) -> Vec<(String, bool)> {
    parse_surface_snapshot(payload).modules
}

pub fn patch_json_modules_set(name: &str, enabled: bool, client_id: Option<&str>) -> String {
    #[derive(Serialize)]
    struct Row<'a> {
        op: &'static str,
        name: &'a str,
        enabled: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_id: Option<&'a str>,
    }
    serde_json::to_string(&vec![Row {
        op: "modules_set",
        name,
        enabled,
        client_id,
    }])
    .unwrap_or_else(|_| "[]".to_string())
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "snapshot",
            description:
                "JSON SurfaceSnapshot (modules, revision, extra.navigation_registry); bump revision on patch",
            run: snapshot,
        },
        ModuleCommand {
            name: "patch",
            description:
                "Apply SurfacePatch JSON array (modules_set + optional client_id); shared host state for multi-client",
            run: patch,
        },
    ]
}
