use std::sync::Arc;

use crate::config::modules::ModulesConfig;
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

/// Set the config directory path. Must be called before any other API on iOS.
/// Desktop callers skip this — $HOME is used by default.
#[uniffi::export]
pub fn set_config_root_path(path: String) {
    crate::config::set_config_root(std::path::PathBuf::from(path));
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
    let result = if enabled {
        cfg.enable_with_requirements(&name)
    } else {
        cfg.set_module_state(&name, false)
    };
    match result {
        Ok(()) => match cfg.save() {
            Ok(()) => format!("Module {name} {}", if enabled { "enabled" } else { "disabled" }),
            Err(e) => format!("Error saving config: {e}"),
        },
        Err(e) => e,
    }
}

/// Returns the current platform name ("ios", "macos", "linux", "windows", "unknown").
#[uniffi::export]
pub fn platform_name() -> String {
    use crate::platform::PlatformInfo;
    crate::platform::current().name().to_string()
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

    pub fn execute(&self, token: String, args: Vec<String>) -> String {
        execute_command(token, args, ExecutionContextFfi::default())
    }

    pub fn execute_remote(&self, token: String, args: Vec<String>, net_as: String) -> String {
        execute_command(
            token,
            args,
            ExecutionContextFfi { net_as: Some(net_as), net_timeout_ms: None },
        )
    }
}
