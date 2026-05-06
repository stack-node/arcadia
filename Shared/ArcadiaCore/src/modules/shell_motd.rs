//! Banner/MOTD inspired by neofetch/fastfetch-style distro greetings (e.g. CachyOS).

use crate::modules::{ExecutionContext, ModuleCommand};
use std::fmt::Write;

pub const NAME: &str = "shell-motd";

/// ASCII block for the left column (fixed width for alignment with info lines).
const ART: &[&str] = &[
    r"      ___                          ",
    r"     /   \   __ _  __ _ _ __ ___   ",
    r"    / /\ \ / _` |/ _` | '__/ _ \  ",
    r"   / /_/ | | (_| | (_| | | | (_) | ",
    r"  /___,'\_\\__, |\__, |_|  \___/  ",
    r"           |___/ |___/             ",
];

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into())
}

fn machine_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name") {
            let s = s.trim();
            if !s.is_empty() && s != "Default string" {
                return s.to_string();
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(m) = run_cmd("sysctl", &["-n", "hw.model"]) {
            return m;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["computersystem", "get", "model"]) {
            let line = o.lines().nth(1).unwrap_or("").trim();
            if !line.is_empty() && !line.eq_ignore_ascii_case("model") {
                return line.to_string();
            }
        }
    }
    hostname_str()
}

fn hostname_str() -> String {
    #[cfg(target_os = "windows")]
    {
        return std::env::var("COMPUTERNAME").unwrap_or_else(|_| "windows".into());
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(h) = std::fs::read_to_string("/etc/hostname") {
            let h = h.lines().next().unwrap_or("").trim();
            if !h.is_empty() {
                return h.to_string();
            }
        }
        run_cmd("hostname", &[]).unwrap_or_else(|| "localhost".into())
    }
}

fn os_pretty() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(txt) = std::fs::read_to_string("/etc/os-release") {
            for line in txt.lines() {
                if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                    let v = rest.trim().trim_matches('"').trim_matches('\'');
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
        return "Linux".into();
    }
    #[cfg(target_os = "macos")]
    {
        let name = run_cmd("sw_vers", &["-productName"]).unwrap_or_else(|| "macOS".into());
        let ver = run_cmd("sw_vers", &["-productVersion"]).unwrap_or_default();
        if ver.is_empty() {
            name
        } else {
            format!("{name} {ver}")
        }
    }
    #[cfg(target_os = "windows")]
    {
        run_cmd("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows".into())
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )))]
    {
        "Arcadia".into()
    }
}

fn kernel_line() -> String {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let sys = run_cmd("uname", &["-s"]).unwrap_or_else(|| "Unix".into());
        let rel = run_cmd("uname", &["-r"]).unwrap_or_default();
        return if rel.is_empty() {
            sys
        } else {
            format!("{sys} {rel}")
        };
    }
    #[cfg(target_os = "windows")]
    {
        return run_cmd("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows NT".into());
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )))]
    {
        "unknown".into()
    }
}

fn uptime_line() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Some(u) = run_cmd("uptime", &["-p"]) {
            return u;
        }
    }
    if let Some(u) = run_cmd("uptime", &[]) {
        return u;
    }
    "n/a".into()
}

fn shell_env() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| std::env::var("COMSPEC").unwrap_or_else(|_| "n/a".into()))
}

fn desktop_env() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "n/a".into())
}

fn cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if let Some(m) = line.strip_prefix("model name\t: ") {
                    return m.trim().to_string();
                }
                if let Some(m) = line.strip_prefix("model name : ") {
                    return m.trim().to_string();
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(m) = run_cmd("sysctl", &["-n", "machdep.cpu.brand_string"]) {
            return m;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["cpu", "get", "name"]) {
            let line = o.lines().nth(1).unwrap_or("").trim();
            if !line.is_empty() {
                return line.to_string();
            }
        }
    }
    "CPU".into()
}

fn memory_line() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(txt) = std::fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = None::<u64>;
            for line in txt.lines() {
                if let Some(n) = line.strip_prefix("MemTotal:") {
                    total_kb = n.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
                }
                if let Some(n) = line.strip_prefix("MemAvailable:") {
                    avail_kb = n.split_whitespace().next().and_then(|s| s.parse().ok());
                }
            }
            if total_kb > 0 {
                let avail = avail_kb.unwrap_or(0);
                let used = total_kb.saturating_sub(avail);
                let pct = 100.0 * used as f64 / total_kb as f64;
                return format!(
                    "{} MiB / {} MiB ({pct:.0}%)",
                    used / 1024,
                    total_kb / 1024
                );
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(bytes) = run_cmd("sysctl", &["-n", "hw.memsize"]) {
            if let Ok(b) = bytes.parse::<u64>() {
                return format!("{} GiB total", b / (1024 * 1024 * 1024));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = run_cmd("wmic", &["computersystem", "get", "TotalPhysicalMemory"]) {
            if let Some(line) = o.lines().nth(1) {
                if let Ok(bytes) = line.trim().parse::<u64>() {
                    return format!("{} GiB total", bytes / (1024 * 1024 * 1024));
                }
            }
        }
    }
    "n/a".into()
}

fn palette_footer() -> String {
    "● ● ● ● ● ● ● ●".into()
}

fn gather_right_column() -> Vec<String> {
    let host = hostname_str();
    let head = format!("{}@{}", username(), host);
    let sep: String = std::iter::repeat('-').take(head.chars().count().min(42)).collect();

    let mut lines = vec![head, sep];

    let pairs: [(&str, String); 9] = [
        ("OS", os_pretty()),
        ("Host", machine_model()),
        ("Kernel", kernel_line()),
        ("Uptime", uptime_line()),
        ("Shell", shell_env()),
        ("DE", desktop_env()),
        ("Terminal", "Arcadia".into()),
        ("CPU", cpu_model()),
        ("Memory", memory_line()),
    ];

    for (k, v) in pairs {
        let mut line = String::new();
        let _ = write!(line, "{k}: {v}");
        lines.push(line);
    }

    lines.push(String::new());
    lines.push(palette_footer());
    lines
}

fn merge_columns(left: &[&str], right: &[String]) -> Vec<String> {
    let left_pad = left.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let n = left.len().max(right.len());
    let gap = "    ";
    let mut out = Vec::with_capacity(n + 1);
    for i in 0..n {
        let l = left
            .get(i)
            .copied()
            .unwrap_or("")
            .chars()
            .chain(std::iter::repeat(' '))
            .take(left_pad)
            .collect::<String>();
        let r = right.get(i).cloned().unwrap_or_default();
        out.push(format!("{l}{gap}{r}"));
    }
    out
}

fn show(_args: &[&str], _ctx: &ExecutionContext) -> String {
    motd_string()
}

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "show",
        description: "print Arcadia-style system banner (MOTD)",
        run: show,
    }]
}

pub fn motd_string() -> String {
    motd_lines().join("\n")
}

/// Lines shown when the shell opens with `shell-motd` enabled (plain text; GUI uses a single color).
pub fn motd_lines() -> Vec<String> {
    let mut lines = merge_columns(ART, &gather_right_column());
    lines.push(String::new());
    lines.push("Arcadia Terminal ready.".into());
    lines
}
