use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "late.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateConfig {
    pub server_url: String,
    pub auth_token: String,
    pub username: String,
    pub default_room: u8,
}

impl Default for LateConfig {
    fn default() -> Self {
        Self {
            server_url: "https://late.sh".to_string(),
            auth_token: String::new(),
            username: String::new(),
            default_room: 1,
        }
    }
}

impl ConfigFile for LateConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}
