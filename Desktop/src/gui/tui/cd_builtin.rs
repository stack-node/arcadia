//! Infer cwd after a trivial `cd` when PTY foreground pgid is unavailable (`tcgetpgrp`).

use std::path::{Component, Path, PathBuf};

pub(crate) fn resolve_simple_cd(base: &Path, command: &str) -> Option<PathBuf> {
    let operand = parse_cd_operand(command)?;
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_cd_operand(base, home.as_deref(), operand)
}

fn parse_cd_operand(cmd: &str) -> Option<Option<String>> {
    let cmd = cmd.trim();
    if cmd == "cd" {
        return Some(None);
    }
    const PREFIX: &str = "cd ";
    if !cmd.starts_with(PREFIX) {
        return None;
    }
    let arg = cmd[PREFIX.len()..].trim();
    if arg.is_empty() {
        return Some(None);
    }
    if arg.contains(';') || arg.contains("&&") || arg.contains('|') || arg.contains('\n') {
        return None;
    }
    if arg.split_whitespace().nth(1).is_some() {
        return None;
    }
    Some(Some(arg.to_string()))
}

fn resolve_cd_operand(
    base: &Path,
    home: Option<&Path>,
    operand: Option<String>,
) -> Option<PathBuf> {
    let raw = match operand {
        None => home?.to_path_buf(),
        Some(arg) => {
            if arg == "-" {
                return None;
            }
            expand_path(base, home, &arg)
        }
    };
    let normalized = lexical_normalize(raw);
    normalized.exists().then_some(normalized)
}

fn expand_path(base: &Path, home: Option<&Path>, arg: &str) -> PathBuf {
    if let Some(rest) = arg.strip_prefix('~') {
        return match home {
            Some(h) => {
                if rest.is_empty() {
                    h.to_path_buf()
                } else if rest.starts_with('/') {
                    h.join(rest.trim_start_matches('/'))
                } else {
                    h.join(rest)
                }
            }
            None => PathBuf::from(arg),
        };
    }
    if arg.starts_with('/') {
        PathBuf::from(arg)
    } else {
        base.join(arg)
    }
}

fn lexical_normalize(path: PathBuf) -> PathBuf {
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    let mut absolute = false;

    for comp in path.components() {
        match comp {
            Component::Prefix(p) => stack.push(p.as_os_str().to_owned()),
            Component::RootDir => {
                absolute = true;
                stack.clear();
                stack.push(comp.as_os_str().to_owned());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if absolute && stack.len() <= 1 {
                    continue;
                }
                let _ = stack.pop();
            }
            Component::Normal(o) => stack.push(o.to_owned()),
        }
    }

    let mut out = PathBuf::new();
    for s in stack {
        out.push(s);
    }
    if out.as_os_str().is_empty() {
        PathBuf::from("/")
    } else {
        out
    }
}
