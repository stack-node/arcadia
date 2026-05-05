use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::config::{config_file_path, config_root_dir};

const FILE_NAME: &str = "commandline.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandlineConfig {
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_color: String,
    pub output_color: String,
}

impl Default for CommandlineConfig {
    fn default() -> Self {
        Self {
            input_symbol: ">".to_string(),
            output_symbol: "~".to_string(),
            input_color: "magenta".to_string(),
            output_color: "cyan".to_string(),
        }
    }
}

impl CommandlineConfig {
    pub fn load_or_create() -> io::Result<Self> {
        let root = config_root_dir()?;
        fs::create_dir_all(&root)?;

        let path = config_file_path(FILE_NAME)?;

        if !path.exists() {
            let default = Self::default();
            let content = toml::to_string_pretty(&default).map_err(io::Error::other)?;
            fs::write(&path, content)?;
            return Ok(default);
        }

        let content = fs::read_to_string(&path)?;
        toml::from_str::<Self>(&content).map_err(io::Error::other)
    }

    pub fn input_ansi_code(&self) -> &'static str {
        color_to_ansi(&self.input_color)
    }

    pub fn output_ansi_code(&self) -> &'static str {
        color_to_ansi(&self.output_color)
    }
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
