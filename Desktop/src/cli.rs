use std::borrow::Cow::{self, Borrowed, Owned};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{OnceLock, RwLock};

use arcadia_core::config::commandline::CommandlineConfig;
use arcadia_core::config::modules::ModulesConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::platform;
use arcadia_core::platform::PlatformInfo;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};

struct CommandSpec {
    name: &'static str,
    aliases: &'static [&'static str],
    subcommands: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct ConfigProviderSpec {
    name: &'static str,
    list_keys: fn() -> Result<Vec<String>, String>,
    get_value: fn(&str) -> Result<String, String>,
    set_value: fn(&str, &str) -> Result<(), String>,
    reset: fn(Option<&str>) -> Result<(), String>,
    ensure_exists: fn() -> Result<(), String>,
    file_path: fn() -> Result<PathBuf, String>,
}

const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "help",
        aliases: &[],
        subcommands: &[],
    },
    CommandSpec {
        name: "ping",
        aliases: &[],
        subcommands: &[],
    },
    CommandSpec {
        name: "configuration",
        aliases: &["config", "cfg"],
        subcommands: &["show", "get", "set", "reset"],
    },
    CommandSpec {
        name: "module",
        aliases: &[],
        subcommands: &["enable", "disable"],
    },
    CommandSpec {
        name: "quit",
        aliases: &[],
        subcommands: &[],
    },
];

static CONFIG_PROVIDERS: &[ConfigProviderSpec] = &[
    ConfigProviderSpec {
        name: "commandline",
        list_keys: commandline_keys,
        get_value: commandline_get,
        set_value: commandline_set,
        reset: commandline_reset,
        ensure_exists: commandline_ensure_exists,
        file_path: commandline_path,
    },
    ConfigProviderSpec {
        name: "modules",
        list_keys: modules_keys,
        get_value: modules_get,
        set_value: modules_set,
        reset: modules_reset,
        ensure_exists: modules_ensure_exists,
        file_path: modules_path,
    },
];

pub enum CommandResult {
    Continue,
    Quit,
}

#[derive(Default)]
struct CliHelper;

impl Helper for CliHelper {}
impl Validator for CliHelper {}

impl Hinter for CliHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos != line.len() {
            return None;
        }

        let (start, suggestions) = completion_candidates(line, pos);
        let typed = &line[start..pos];
        if typed.trim().is_empty() {
            return None;
        }

        suggestions
            .iter()
            .find(|candidate| candidate.starts_with(typed) && candidate.as_str() != typed)
            .map(|candidate| candidate[typed.len()..].to_string())
    }
}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let (start, suggestions) = completion_candidates(line, pos);
        let prefix = line[start..pos].to_ascii_lowercase();

        let suggestions = suggestions
            .into_iter()
            .filter(|candidate| candidate.starts_with(&prefix))
            .map(|candidate| Pair {
                display: candidate.to_string(),
                replacement: candidate.to_string(),
            })
            .collect::<Vec<_>>();

        Ok((start, suggestions))
    }
}

impl Highlighter for CliHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[90m{hint}\x1b[0m"))
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Borrowed(line)
    }
}

fn settings_lock() -> &'static RwLock<CommandlineConfig> {
    static SETTINGS: OnceLock<RwLock<CommandlineConfig>> = OnceLock::new();
    SETTINGS.get_or_init(|| {
        let config = match CommandlineConfig::load_or_create() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to load commandline config; using defaults: {err}");
                CommandlineConfig::default()
            }
        };
        for warning in config.color_warnings() {
            eprintln!("Warning: {warning}");
        }
        config.into()
    })
}

fn settings() -> CommandlineConfig {
    settings_lock()
        .read()
        .map(|cfg| cfg.clone())
        .unwrap_or_else(|_| CommandlineConfig::default())
}

fn root_command_tokens() -> Vec<String> {
    let mut tokens = COMMAND_SPECS
        .iter()
        .flat_map(|spec| std::iter::once(spec.name).chain(spec.aliases.iter().copied()))
        .map(str::to_string)
        .collect::<Vec<_>>();
    tokens.extend(modules::enabled_module_names());
    tokens.extend(modules::enabled_command_tokens());
    tokens
}

fn resolve_command(command: &str) -> Option<&'static CommandSpec> {
    COMMAND_SPECS
        .iter()
        .find(|spec| spec.name == command || spec.aliases.contains(&command))
}

fn commandline_keys() -> Result<Vec<String>, String> {
    Ok(vec![
        "input_symbol".to_string(),
        "output_symbol".to_string(),
        "input_color".to_string(),
        "output_color".to_string(),
        "clear_on_start".to_string(),
    ])
}

fn modules_keys() -> Result<Vec<String>, String> {
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    Ok(cfg.modules.keys().cloned().collect())
}

fn module_set_state(module_name: &str, enabled: bool, with_requirements: bool) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    if enabled && with_requirements {
        cfg.enable_with_requirements(module_name)?;
    } else {
        cfg.set_module_state(module_name, enabled)?;
    }
    cfg.save().map_err(|err| err.to_string())
}

fn commandline_get_value(cfg: &CommandlineConfig, key: &str) -> Option<String> {
    match key {
        "input_symbol" => Some(cfg.input_symbol.clone()),
        "output_symbol" => Some(cfg.output_symbol.clone()),
        "input_color" => Some(cfg.input_color.clone()),
        "output_color" => Some(cfg.output_color.clone()),
        "clear_on_start" => Some(cfg.clear_on_start.to_string()),
        _ => None,
    }
}

fn commandline_get(key: &str) -> Result<String, String> {
    let cfg = settings();
    commandline_get_value(&cfg, key).ok_or_else(|| "Unknown config key".to_string())
}

fn commandline_set(key: &str, value: &str) -> Result<(), String> {
    let mut guard = settings_lock()
        .write()
        .map_err(|_| "Failed to update config: settings lock poisoned".to_string())?;

    let applied = match key {
        "input_symbol" => {
            guard.input_symbol = value.to_string();
            true
        }
        "output_symbol" => {
            guard.output_symbol = value.to_string();
            true
        }
        "input_color" => {
            guard.input_color = value.to_string();
            true
        }
        "output_color" => {
            guard.output_color = value.to_string();
            true
        }
        "clear_on_start" => match value.to_ascii_lowercase().as_str() {
            "true" => {
                guard.clear_on_start = true;
                true
            }
            "false" => {
                guard.clear_on_start = false;
                true
            }
            _ => return Err("clear_on_start must be true or false".to_string()),
        },
        _ => return Err("Unknown config key".to_string()),
    };

    if applied {
        guard
            .save()
            .map_err(|err| format!("Config updated in-memory, save failed: {err}"))?;
    }
    Ok(())
}

fn commandline_reset(target: Option<&str>) -> Result<(), String> {
    let mut guard = settings_lock()
        .write()
        .map_err(|_| "Failed to reset config: settings lock poisoned".to_string())?;
    let defaults = CommandlineConfig::default();

    match target {
        None => *guard = defaults,
        Some("input_symbol") => guard.input_symbol = defaults.input_symbol,
        Some("output_symbol") => guard.output_symbol = defaults.output_symbol,
        Some("input_color") => guard.input_color = defaults.input_color,
        Some("output_color") => guard.output_color = defaults.output_color,
        Some("clear_on_start") => guard.clear_on_start = defaults.clear_on_start,
        Some(_) => return Err("Unknown config key".to_string()),
    }

    guard
        .save()
        .map_err(|err| format!("Config reset in-memory, save failed: {err}"))
}

fn commandline_ensure_exists() -> Result<(), String> {
    CommandlineConfig::load_or_create()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn commandline_path() -> Result<PathBuf, String> {
    CommandlineConfig::file_path().map_err(|err| err.to_string())
}

fn modules_get(key: &str) -> Result<String, String> {
    let cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    cfg.modules
        .get(key)
        .map(|enabled| enabled.to_string())
        .ok_or_else(|| "Unknown module key".to_string())
}

fn modules_set(key: &str, value: &str) -> Result<(), String> {
    let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
    let parsed = match value.to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return Err("Module value must be true or false".to_string()),
    };

    cfg.set_module_state(key, parsed)?;
    cfg.save().map_err(|err| err.to_string())
}

fn modules_reset(target: Option<&str>) -> Result<(), String> {
    match target {
        None => ModulesConfig::default().save().map_err(|err| err.to_string()),
        Some(key) => {
            let defaults = ModulesConfig::default();
            let default_value = defaults
                .modules
                .get(key)
                .copied()
                .ok_or_else(|| "Unknown module key".to_string())?;

            let mut cfg = ModulesConfig::load_or_create().map_err(|err| err.to_string())?;
            cfg.set_module_state(key, default_value)?;
            cfg.save().map_err(|err| err.to_string())
        }
    }
}

fn modules_ensure_exists() -> Result<(), String> {
    ModulesConfig::load_or_create()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn modules_path() -> Result<PathBuf, String> {
    ModulesConfig::file_path().map_err(|err| err.to_string())
}

fn provider_names() -> Vec<String> {
    CONFIG_PROVIDERS
        .iter()
        .map(|provider| provider.name.to_string())
        .collect()
}

fn resolve_provider(name: &str) -> Option<ConfigProviderSpec> {
    CONFIG_PROVIDERS
        .iter()
        .find(|provider| provider.name == name)
        .copied()
}

fn scoped_key_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    for provider in CONFIG_PROVIDERS {
        if let Ok(keys) = (provider.list_keys)() {
            for key in keys {
                candidates.push(format!("{}.{}", provider.name, key));
            }
        }
    }
    candidates
}

fn normalize_command(command: &str) -> String {
    resolve_command(command)
        .map(|spec| spec.name.to_string())
        .unwrap_or_else(|| command.to_string())
}

fn parse_execution_context(parts: &[String]) -> Result<(Vec<String>, modules::ExecutionContext), String> {
    let mut cleaned = Vec::new();
    let mut net_as: Option<String> = None;
    let mut net_timeout_ms: Option<u64> = None;
    let mut i = 0;

    while i < parts.len() {
        if parts[i] == "--net:as" {
            let Some(value) = parts.get(i + 1) else {
                return Err("Usage: --net:as lan:<host/ip/alias>".to_string());
            };
            if !value.starts_with("lan:") {
                return Err(
                    "Unsupported --net:as target. Use lan:<host/ip/alias> (wan: coming later)"
                        .to_string(),
                );
            };
            net_as = Some(value.clone());
            i += 2;
            continue;
        }
        if parts[i] == "--net:timeout" {
            let Some(value) = parts.get(i + 1) else {
                return Err("Usage: --net:timeout <milliseconds>".to_string());
            };
            let parsed = value
                .parse::<u64>()
                .map_err(|_| "Invalid --net:timeout value. Use an integer in milliseconds".to_string())?;
            net_timeout_ms = Some(parsed);
            i += 2;
            continue;
        }
        cleaned.push(parts[i].clone());
        i += 1;
    }

    Ok((
        cleaned,
        modules::ExecutionContext {
            net_as,
            net_timeout_ms,
        },
    ))
}

fn completion_candidates(line: &str, pos: usize) -> (usize, Vec<String>) {
    let head = &line[..pos];
    let ends_with_space = head.chars().last().is_some_and(char::is_whitespace);
    let tokens = head.split_whitespace().collect::<Vec<_>>();

    if tokens.is_empty() {
        return (0, root_command_tokens());
    }

    if tokens.len() == 1 && !ends_with_space {
        let start = head.rfind(char::is_whitespace).map_or(0, |idx| idx + 1);
        return (start, root_command_tokens());
    }

    let command = normalize_command(tokens[0]);
    let active_index = if ends_with_space {
        tokens.len()
    } else {
        tokens.len().saturating_sub(1)
    };
    let start = head.rfind(char::is_whitespace).map_or(0, |idx| idx + 1);

    let suggestions = match command.as_str() {
        "configuration" => match active_index {
            1 => resolve_command("configuration")
                .map(|spec| spec.subcommands.iter().map(|v| (*v).to_string()).collect())
                .unwrap_or_default(),
            2 => match tokens.get(1).copied() {
                Some("show") => provider_names(),
                Some("get") | Some("set") | Some("reset") => provider_names()
                    .into_iter()
                    .chain(commandline_keys().unwrap_or_default())
                    .chain(scoped_key_candidates())
                    .collect(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        "module" => match active_index {
            1 => modules_keys().unwrap_or_default(),
            2 => vec!["enable".to_string(), "disable".to_string()],
            3 if tokens.get(2).copied() == Some("enable") => vec!["-requirements".to_string()],
            _ => Vec::new(),
        },
        other => match active_index {
            1 => modules::enabled_module_command_names(other),
            _ => Vec::new(),
        },
    };

    (start, suggestions)
}

pub fn print_response(message: &str) {
    let cfg = settings();
    println!("{}{}\x1b[0m {message}", cfg.output_ansi_code(), cfg.output_symbol);
}

pub fn print_startup(mode: &str) {
    if settings().clear_on_start {
        // Clear terminal and move cursor to top-left for a clean boot screen.
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
    }

    println!("Arcadia base app");
    println!("Detected platform: {}", platform::current().name());
    println!("Mode: {mode}");
    println!("Status: bootstrap complete");
}

fn print_help() {
    print_response("Available commands:");
    for spec in COMMAND_SPECS {
        match spec.name {
            "help" => print_response("- help: show this help message"),
            "ping" => print_response("- ping: respond with pong"),
            "quit" => print_response("- quit: exit Arcadia"),
            "configuration" => {
                print_response("- configuration <name>: open config file in default editor");
                print_response(
                    "- configuration [show|get|set|reset] ...: manage commandline config",
                );
                if !spec.aliases.is_empty() {
                    let aliases = spec.aliases.join(" -> ");
                    print_response(&format!("- aliases: {aliases} -> configuration"));
                }
            }
            "module" => {
                print_response("- module <name> enable|disable: toggle a module");
                print_response(
                    "- module <name> enable -requirements: enable module and required dependencies",
                );
            }
            _ => {}
        }
    }

    let module_command_lines = modules::enabled_command_help_lines();
    if !module_command_lines.is_empty() {
        print_response("- enabled module commands:");
        for line in module_command_lines {
            print_response(&line);
        }
    }
    print_response("- global flags: --net:as lan:<host/ip/alias> | --net:timeout <milliseconds>");
}

fn handle_module(parts: &[&str]) {
    match parts {
        ["module", module_name, "enable"] => match module_set_state(module_name, true, false) {
            Ok(_) => print_response(&format!("Module {module_name} enabled")),
            Err(err) => print_response(&err),
        },
        ["module", module_name, "enable", "-requirements"] => {
            match module_set_state(module_name, true, true) {
                Ok(_) => print_response(&format!(
                    "Module {module_name} enabled (requirements enabled)"
                )),
                Err(err) => print_response(&err),
            }
        }
        ["module", module_name, "disable"] => match module_set_state(module_name, false, false) {
            Ok(_) => print_response(&format!("Module {module_name} disabled")),
            Err(err) => print_response(&err),
        },
        ["module"] => {
            print_response("Usage: module <name> enable [-requirements]|disable");
            match modules_keys() {
                Ok(keys) => {
                    if !keys.is_empty() {
                        print_response("Available modules:");
                        for key in keys {
                            print_response(&format!("- {key}"));
                        }
                    }
                }
                Err(err) => print_response(&format!("Failed to list modules: {err}")),
            }
        }
        _ => print_response("Usage: module <name> enable [-requirements]|disable"),
    }
}

fn print_available_configs() {
    print_response("Available configs:");
    for name in provider_names() {
        print_response(&format!("- {name}"));
    }
}

fn show_config_keys(config_name: &str) {
    let Some(provider) = resolve_provider(config_name) else {
        print_response("Unknown config");
        return;
    };

    match (provider.list_keys)() {
        Ok(keys) => {
            print_response(&format!("{} keys:", provider.name));
            for key in keys {
                print_response(&format!("- {key}"));
            }
        }
        Err(err) => {
            print_response(&format!("Failed to load {} config: {err}", provider.name));
        }
    }
}

fn config_get(key: &str) {
    match commandline_get(key) {
        Ok(value) => print_response(&format!("{key} = {value}")),
        Err(err) => print_response(&err),
    }
}

fn config_get_scoped(reference: &str) {
    let Some((config_name, key)) = reference.split_once('.') else {
        print_response("Use scoped format: <config>.<key> (example: commandline.clear_on_start)");
        return;
    };

    let Some(provider) = resolve_provider(config_name) else {
        print_response("Unknown config");
        return;
    };

    match (provider.get_value)(key) {
        Ok(value) => print_response(&format!("{config_name}.{key} = {value}")),
        Err(err) => print_response(&err),
    }
}

fn config_set(key: &str, value: &str) {
    match commandline_set(key, value) {
        Ok(_) => print_response("Config updated"),
        Err(err) => print_response(&err),
    }
}

fn config_set_scoped(reference: &str, value: &str) {
    let Some((config_name, key)) = reference.split_once('.') else {
        print_response("Use scoped format: <config>.<key> (example: commandline.clear_on_start)");
        return;
    };

    let Some(provider) = resolve_provider(config_name) else {
        print_response("Unknown config");
        return;
    };

    match (provider.set_value)(key, value) {
        Ok(_) => print_response("Config updated"),
        Err(err) => print_response(&err),
    }
}

fn config_reset() {
    match commandline_reset(None) {
        Ok(_) => print_response("Config reset to defaults"),
        Err(err) => print_response(&err),
    }
}

fn config_reset_scoped(reference: &str) {
    let (config_name, target_key) = match reference.split_once('.') {
        Some((name, key)) => (name, Some(key)),
        None => (reference, None),
    };

    let Some(provider) = resolve_provider(config_name) else {
        print_response("Unknown config");
        return;
    };

    match (provider.reset)(target_key) {
        Ok(_) => {
            if target_key.is_some() {
                print_response("Config key reset to default");
            } else {
                print_response("Config reset to defaults");
            }
        }
        Err(err) => print_response(&err),
    }
}

fn open_config(config_name: &str) {
    let Some(provider) = resolve_provider(config_name) else {
        print_response("Unknown config");
        return;
    };

    if let Err(err) = (provider.ensure_exists)() {
        print_response(&format!("Failed to create {} config: {err}", provider.name));
        return;
    }

    let path = match (provider.file_path)() {
        Ok(path) => path,
        Err(err) => {
            print_response(&format!(
                "Failed to resolve {} config path: {err}",
                provider.name
            ));
            return;
        }
    };

    let status = {
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(&path).status()
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(&path).status()
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .status()
        }
    };

    match status {
        Ok(exit) if exit.success() => print_response(&format!("Opened {}", path.display())),
        Ok(exit) => print_response(&format!(
            "Failed to open {} (exit code: {:?})",
            path.display(),
            exit.code()
        )),
        Err(err) => print_response(&format!("Failed to launch editor for {}: {err}", path.display())),
    }
}

fn handle_configuration(parts: &[&str]) {
    match parts {
        ["configuration"] => print_available_configs(),
        ["configuration", "show"] => print_available_configs(),
        ["configuration", "show", config_name] => show_config_keys(config_name),
        ["configuration", "get", key] if key.contains('.') => config_get_scoped(key),
        ["configuration", "get", key] => config_get(key),
        ["configuration", "set", key, value] if key.contains('.') => config_set_scoped(key, value),
        ["configuration", "set", key, value] => config_set(key, value),
        ["configuration", "reset", target] => config_reset_scoped(target),
        ["configuration", "reset"] => config_reset(),
        ["configuration", name] => open_config(name),
        _ => {
            print_response("Usage: configuration <name> | configuration [show|get <key>|get <config>.<key>|set <key> <value>|set <config>.<key> <value>|reset|reset <config>|reset <config>.<key>]");
            print_response(
                "Keys: input_symbol, output_symbol, input_color, output_color, clear_on_start",
            );
        }
    }
}

fn editor() -> Editor<CliHelper, DefaultHistory> {
    let config = Config::builder()
        .history_ignore_dups(true)
        .expect("history_ignore_dups is always configurable")
        .completion_type(CompletionType::List)
        .build();

    let mut editor =
        Editor::<CliHelper, DefaultHistory>::with_config(config).expect("editor setup failed");
    editor.set_helper(Some(CliHelper));
    editor
}

fn read_command(editor: &mut Editor<CliHelper, DefaultHistory>) -> io::Result<Option<String>> {
    let cfg = settings();
    let prompt = format!(
        "\x01{}\x02{}\x01\x1b[0m\x02 ",
        cfg.input_ansi_code(),
        cfg.input_symbol
    );
    match editor.readline(&prompt) {
        Ok(line) => {
            if !line.trim().is_empty() {
                let _ = editor.add_history_entry(line.as_str());
            }
            Ok(Some(line))
        }
        Err(ReadlineError::Interrupted) => Ok(Some(String::new())),
        Err(ReadlineError::Eof) => Ok(None),
        Err(err) => Err(io::Error::other(err)),
    }
}

pub fn start_loop(quit: impl FnOnce() + Copy) {
    let mut editor = editor();

    loop {
        match read_command(&mut editor) {
            Ok(None) => break,
            Ok(Some(line)) => {
                if let CommandResult::Quit = handle(&line) {
                    quit();
                    break;
                }
            }
            Err(err) => {
                eprintln!("CLI input error: {err}");
                break;
            }
        }
    }
}

pub fn handle(input: &str) -> CommandResult {
    let trimmed = input.trim();
    let mut parts = trimmed
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let (parsed_parts, exec_ctx) = match parse_execution_context(&parts) {
        Ok(value) => value,
        Err(err) => {
            print_response(&err);
            return CommandResult::Continue;
        }
    };
    parts = parsed_parts;

    if let Some(first) = parts.first_mut() {
        *first = normalize_command(first);
    }

    if !parts.is_empty() && parts[0] == "configuration" {
        let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
        handle_configuration(&part_refs);
        return CommandResult::Continue;
    }
    if !parts.is_empty() && parts[0] == "module" {
        let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
        handle_module(&part_refs);
        return CommandResult::Continue;
    }
    if let Some(first) = parts.first().map(String::as_str) {
        if first.contains('.') {
            let args = parts
                .iter()
                .skip(1)
                .map(String::as_str)
                .collect::<Vec<_>>();
            match modules::execute_command(first, &args, &exec_ctx) {
                Ok(Some(message)) => {
                    print_response(&message);
                    return CommandResult::Continue;
                }
                Ok(None) => {}
                Err(err) => {
                    print_response(&err);
                    return CommandResult::Continue;
                }
            }
        }
    }
    if parts.len() >= 2 {
        let composed = format!("{}.{}", parts[0], parts[1]);
        let args = parts
            .iter()
            .skip(2)
            .map(String::as_str)
            .collect::<Vec<_>>();
        match modules::execute_command(&composed, &args, &exec_ctx) {
            Ok(Some(message)) => {
                print_response(&message);
                return CommandResult::Continue;
            }
            Ok(None) => {}
            Err(err) => {
                print_response(&err);
                return CommandResult::Continue;
            }
        }
    }

    match parts.first().map(String::as_str).unwrap_or("") {
        "help" => {
            print_help();
            CommandResult::Continue
        }
        "ping" => {
            print_response("pong");
            CommandResult::Continue
        }
        "quit" => CommandResult::Quit,
        "" => CommandResult::Continue,
        _ => {
            print_response(&format!("Unknown command: {trimmed}"));
            CommandResult::Continue
        }
    }
}
