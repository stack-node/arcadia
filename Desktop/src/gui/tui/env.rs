//! PATH (and related) fixes for GUI-spawned shells — inherited env is often minimal.

use portable_pty::CommandBuilder;
use std::collections::HashSet;

#[cfg(target_os = "macos")]
fn macos_path_helper_segments() -> Vec<String> {
    let Ok(output) = std::process::Command::new("/usr/libexec/path_helper").output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("PATH=\"") else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        return rest[..end]
            .split(':')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
    }
    Vec::new()
}

#[cfg(not(target_os = "macos"))]
fn macos_path_helper_segments() -> Vec<String> {
    Vec::new()
}

fn extra_bin_dirs() -> Vec<String> {
    let mut v = Vec::new();
    #[cfg(target_os = "macos")]
    {
        v.push("/opt/homebrew/bin".to_string());
        v.push("/opt/homebrew/sbin".to_string());
        v.push("/usr/local/bin".to_string());
    }
    v.push("/usr/bin".to_string());
    v.push("/bin".to_string());
    v.push("/usr/sbin".to_string());
    v.push("/sbin".to_string());
    if let Ok(home) = std::env::var("HOME") {
        v.push(format!("{home}/.local/bin"));
        v.push(format!("{home}/.cargo/bin"));
        v.push(format!("{home}/.npm-global/bin"));
    }
    v
}

fn split_path(raw: &str) -> Vec<String> {
    raw.split(':')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn merged_path_for_shell() -> String {
    let mut seen = HashSet::<String>::new();
    let mut parts = Vec::new();

    for segment in macos_path_helper_segments()
        .into_iter()
        .chain(extra_bin_dirs())
        .chain(
            std::env::var("PATH")
                .map(|s| split_path(&s))
                .unwrap_or_default(),
        )
    {
        if seen.insert(segment.clone()) {
            parts.push(segment);
        }
    }

    if parts.is_empty() {
        "/usr/bin:/bin:/usr/sbin:/sbin".to_string()
    } else {
        parts.join(":")
    }
}

pub(crate) fn apply_interactive_shell_env(cmd: &mut CommandBuilder) {
    cmd.env("PATH", merged_path_for_shell());
}
