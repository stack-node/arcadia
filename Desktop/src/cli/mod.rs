mod args;
mod completion;
mod config_cmds;
mod module_cmds;

use std::io::{self, Write};
use std::sync::{OnceLock, RwLock};

use arcadia_core::config::commandline::CommandlineConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::platform;
use arcadia_core::platform::PlatformInfo;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};

use args::{normalize_command, parse_execution_context, COMMAND_SPECS};
use completion::CliHelper;
use config_cmds::handle_configuration;
use module_cmds::handle_module;

pub enum CommandResult {
    Continue,
    Quit,
}

pub(super) fn settings_lock() -> &'static RwLock<CommandlineConfig> {
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

pub(super) fn settings() -> CommandlineConfig {
    settings_lock()
        .read()
        .map(|cfg| cfg.clone())
        .unwrap_or_else(|_| CommandlineConfig::default())
}

pub fn print_response(message: &str) {
    let cfg = settings();
    println!(
        "{}{}\x1b[0m {message}",
        cfg.output_ansi_code(),
        cfg.output_symbol
    );
}

pub fn print_startup(mode: &str) {
    if settings().clear_on_start {
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
    let prompt = format!("{}{}\x1b[0m ", cfg.input_ansi_code(), cfg.input_symbol);
    match editor.readline(&prompt) {
        Ok(line) => {
            if !line.trim().is_empty() {
                let _ = editor.add_history_entry(line.as_str());
            }
            Ok(Some(line))
        }
        Err(rustyline::error::ReadlineError::Interrupted) => Ok(Some(String::new())),
        Err(rustyline::error::ReadlineError::Eof) => Ok(None),
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
            let args = parts.iter().skip(1).map(String::as_str).collect::<Vec<_>>();
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
        let args = parts.iter().skip(2).map(String::as_str).collect::<Vec<_>>();
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
