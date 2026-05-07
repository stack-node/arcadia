use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;

use super::config_cmds::modules_keys;

pub fn module_set_state(
    module_name: &str,
    enabled: bool,
    with_requirements: bool,
) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    if enabled && with_requirements {
        cfg.enable_with_requirements(module_name)?;
    } else {
        cfg.set_module_state(module_name, enabled)?;
    }
    cfg.save().map_err(|err| err.to_string())
}

pub fn handle_module(parts: &[&str]) {
    match parts {
        ["module", module_name, "enable"] => match module_set_state(module_name, true, false) {
            Ok(_) => super::print_response(&format!("Module {module_name} enabled")),
            Err(err) => super::print_response(&err),
        },
        ["module", module_name, "enable", "-requirements"] => {
            match module_set_state(module_name, true, true) {
                Ok(_) => super::print_response(&format!(
                    "Module {module_name} enabled (requirements enabled)"
                )),
                Err(err) => super::print_response(&err),
            }
        }
        ["module", module_name, "disable"] => match module_set_state(module_name, false, false) {
            Ok(_) => super::print_response(&format!("Module {module_name} disabled")),
            Err(err) => super::print_response(&err),
        },
        ["module"] => {
            super::print_response("Usage: module <name> enable [-requirements]|disable");
            match modules_keys() {
                Ok(keys) if !keys.is_empty() => {
                    super::print_response("Available modules:");
                    for key in keys {
                        super::print_response(&format!("- {key}"));
                    }
                }
                Ok(_) => {}
                Err(err) => super::print_response(&format!("Failed to list modules: {err}")),
            }
        }
        _ => super::print_response("Usage: module <name> enable [-requirements]|disable"),
    }
}
