use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default_theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_theme: "catppuccin".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lsa")
        .join("config.toml")
}

pub fn load_config() -> Config {
    let config_path = get_config_path();

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(_) => Config::default(),
            },
            Err(_) => Config::default(),
        }
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}