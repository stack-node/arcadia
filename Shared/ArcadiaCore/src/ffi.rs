use std::sync::Arc;

use crate::config::modules::ModulesConfig;
use crate::config::thin_client::ThinClientConfig;
use crate::config::ConfigFile;
use crate::modules;

#[derive(uniffi::Record, Default)]
pub struct ExecutionContextFfi {
    pub net_as: Option<String>,
    pub net_timeout_ms: Option<u64>,
}

#[derive(uniffi::Record)]
pub struct CommandInfo {
    pub token: String,
    pub description: String,
}

#[derive(uniffi::Record)]
pub struct ModuleStatus {
    pub name: String,
    pub enabled: bool,
}

#[derive(uniffi::Record)]
pub struct ModuleToggleResult {
    pub ok: bool,
    pub message: String,
    pub missing_requirements: Vec<String>,
}

#[derive(uniffi::Record)]
pub struct RemoteMirrorDrain {
    pub lines: Vec<String>,
    /// Host handled inbound NODE_EXEC — surfaces showing **local** state should resync from disk / LAN snapshot.
    pub sync_local_surface: bool,
}

/// Set the config directory path. Must be called before any other API on iOS.
/// Desktop callers skip this — $HOME is used by default.
#[uniffi::export]
pub fn set_config_root_path(path: String) {
    crate::config::set_config_root(std::path::PathBuf::from(path));
}

/// Drain mirrored NODE_EXEC transcript lines + host UI sync flag (reload modules when showing local state).
#[uniffi::export]
pub fn drain_remote_mirror_batch() -> RemoteMirrorDrain {
    RemoteMirrorDrain {
        lines: modules::remote_mirror::drain_formatted_mirror_lines(),
        sync_local_surface: modules::remote_mirror::take_host_ui_sync_pending(),
    }
}

/// Execute a command by dot-separated token (e.g. "lan.scan").
/// Returns the result string; errors are embedded in the return value.
#[uniffi::export]
pub fn execute_command(token: String, args: Vec<String>, context: ExecutionContextFfi) -> String {
    let ctx = modules::ExecutionContext {
        net_as: context.net_as,
        net_timeout_ms: context.net_timeout_ms,
    };
    let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();
    match modules::execute_command(&token, &arg_slices, &ctx) {
        Ok(Some(result)) => result,
        Ok(None) => format!("Unknown command: {token}"),
        Err(e) => e,
    }
}

/// List all commands in currently-enabled modules.
#[uniffi::export]
pub fn list_commands() -> Vec<CommandInfo> {
    modules::all_command_entries()
        .into_iter()
        .map(|(token, description)| CommandInfo { token, description })
        .collect()
}

/// List all modules and their enabled state.
#[uniffi::export]
pub fn list_modules() -> Vec<ModuleStatus> {
    ModulesConfig::load_or_create()
        .map(|cfg| {
            cfg.modules
                .into_iter()
                .map(|(name, enabled)| ModuleStatus { name, enabled })
                .collect()
        })
        .unwrap_or_default()
}

/// Enable or disable a named module. Persists to disk. Returns status message.
#[uniffi::export]
pub fn set_module_enabled(name: String, enabled: bool) -> String {
    let mut cfg = match ModulesConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("Error loading config: {e}"),
    };
    let result = cfg.set_module_state(&name, enabled);
    match result {
        Ok(()) => match cfg.save() {
            Ok(()) => {
                modules::surface::bump_surface_revision();
                format!("Module {name} {}", if enabled { "enabled" } else { "disabled" })
            }
            Err(e) => format!("Error saving config: {e}"),
        },
        Err(e) => e,
    }
}

#[uniffi::export]
pub fn set_module_enabled_with_requirements(name: String, enabled: bool) -> String {
    let mut cfg = match ModulesConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("Error loading config: {e}"),
    };
    let result = if enabled {
        cfg.enable_with_requirements(&name)
    } else {
        cfg.set_module_state(&name, false)
    };
    match result {
        Ok(()) => match cfg.save() {
            Ok(()) => {
                modules::surface::bump_surface_revision();
                format!("Module {name} {}", if enabled { "enabled" } else { "disabled" })
            }
            Err(e) => format!("Error saving config: {e}"),
        },
        Err(e) => e,
    }
}

#[uniffi::export]
pub fn probe_module_toggle(name: String, enabled: bool) -> ModuleToggleResult {
    let cfg = match ModulesConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => {
            return ModuleToggleResult {
                ok: false,
                message: format!("Error loading config: {e}"),
                missing_requirements: vec![],
            };
        }
    };
    if !enabled {
        return ModuleToggleResult {
            ok: true,
            message: String::new(),
            missing_requirements: vec![],
        };
    }
    match cfg.missing_requirements_for(&name) {
        Ok(missing) if missing.is_empty() => ModuleToggleResult {
            ok: true,
            message: String::new(),
            missing_requirements: vec![],
        },
        Ok(missing) => ModuleToggleResult {
            ok: false,
            message: format!("Cannot enable {name} without required modules."),
            missing_requirements: missing,
        },
        Err(e) => ModuleToggleResult {
            ok: false,
            message: e,
            missing_requirements: vec![],
        },
    }
}

/// Returns the current platform name ("ios", "macos", "linux", "windows", "unknown").
#[uniffi::export]
pub fn platform_name() -> String {
    use crate::platform::PlatformInfo;
    crate::platform::current().name().to_string()
}

/// Returns the default navigation registry as JSON.
/// This payload is shared by desktop and mobile shells and can later be merged
/// with extension-provided pages/groups at runtime.
#[uniffi::export]
pub fn navigation_registry_json() -> String {
    crate::navigation::default_navigation_registry_json()
}

/// Stable id for this GUI peer (`surface.patch` client_id, logs).
#[uniffi::export]
pub fn thin_client_surface_client_id() -> String {
    ThinClientConfig::load_surface_client_id()
}

#[uniffi::export]
pub fn thin_client_preferred_route_get() -> Option<String> {
    ThinClientConfig::load_or_create()
        .ok()
        .and_then(|c| c.preferred_remote_route.clone())
}

/// Persist default `net_as` route; empty error string on success.
#[uniffi::export]
pub fn thin_client_preferred_route_set(route: Option<String>) -> String {
    match ThinClientConfig::set_preferred_remote_route(route.as_deref()) {
        Ok(()) => String::new(),
        Err(e) => format!("Error saving thin-client preferences: {e}"),
    }
}

/// Start the LAN background service thread. Safe to call multiple times.
#[uniffi::export]
pub fn lan_start() {
    crate::modules::lan::start_service();
}

/// Stop the LAN background service thread.
#[uniffi::export]
pub fn lan_stop() {
    crate::modules::lan::stop_service();
}

#[derive(uniffi::Record)]
pub struct LanServiceInfoFfi {
    pub running: bool,
    pub port: u16,
    pub hostname: String,
    pub module_enabled: bool,
}

/// LAN service status: running state, UDP port, hostname, and module-enabled flag.
#[uniffi::export]
pub fn lan_service_info() -> LanServiceInfoFfi {
    let info = crate::modules::lan::lan_service_info();
    LanServiceInfoFfi {
        running: info.running,
        port: info.port,
        hostname: info.hostname,
        module_enabled: info.module_enabled,
    }
}

/// Object-oriented handle for module and command management.
/// Useful for SwiftUI @StateObject / @Observable patterns.
#[derive(uniffi::Object)]
pub struct ModuleManager;

#[uniffi::export]
impl ModuleManager {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(ModuleManager)
    }

    pub fn list_modules(&self) -> Vec<ModuleStatus> {
        list_modules()
    }

    pub fn list_commands(&self) -> Vec<CommandInfo> {
        list_commands()
    }

    pub fn set_enabled(&self, name: String, enabled: bool) -> String {
        set_module_enabled(name, enabled)
    }

    pub fn set_enabled_with_requirements(&self, name: String, enabled: bool) -> String {
        set_module_enabled_with_requirements(name, enabled)
    }

    pub fn probe_toggle(&self, name: String, enabled: bool) -> ModuleToggleResult {
        probe_module_toggle(name, enabled)
    }

    pub fn execute(&self, token: String, args: Vec<String>) -> String {
        execute_command(token, args, ExecutionContextFfi::default())
    }

    pub fn execute_remote(&self, token: String, args: Vec<String>, net_as: String) -> String {
        execute_command(
            token,
            args,
            ExecutionContextFfi {
                net_as: Some(net_as),
                net_timeout_ms: None,
            },
        )
    }
}
