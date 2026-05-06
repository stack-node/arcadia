use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "commandline.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandlineConfig {
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_color: String,
    pub output_color: String,
    #[serde(default = "default_clear_on_start")]
    pub clear_on_start: bool,
}

impl Default for CommandlineConfig {
    fn default() -> Self {
        Self {
            input_symbol: ">".to_string(),
            output_symbol: "~".to_string(),
            input_color: "magenta".to_string(),
            output_color: "cyan".to_string(),
            clear_on_start: default_clear_on_start(),
        }
    }
}

impl ConfigFile for CommandlineConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}

impl CommandlineConfig {
    pub fn input_ansi_code(&self) -> &'static str {
        color_to_ansi(&self.input_color)
    }

    pub fn output_ansi_code(&self) -> &'static str {
        color_to_ansi(&self.output_color)
    }

    pub fn color_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !is_known_color(&self.input_color) {
            warnings.push(format!(
                "Unknown input_color '{}'; valid: black red green yellow blue magenta cyan white",
                self.input_color
            ));
        }
        if !is_known_color(&self.output_color) {
            warnings.push(format!(
                "Unknown output_color '{}'; valid: black red green yellow blue magenta cyan white",
                self.output_color
            ));
        }
        warnings
    }
}

fn default_clear_on_start() -> bool {
    true
}

fn is_known_color(color: &str) -> bool {
    matches!(
        color.trim().to_ascii_lowercase().as_str(),
        "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white"
    )
}

fn color_to_ansi(color: &str) -> &'static str {
    match color.trim().to_ascii_lowercase().as_str() {
        "black" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        _ => "\x1b[0m",
    }
}
