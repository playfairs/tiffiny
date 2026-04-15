use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub output_directory: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_directory: None,
        }
    }
}

impl Config {
    pub fn get_config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join("tiffiny").join("config.json")
    }

    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if !config_path.exists() {
            return Config::default();
        }

        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn set_output_directory(&mut self, path: &str) {
        self.output_directory = Some(path.to_string());
    }

    pub fn get_output_directory(&self) -> Option<&String> {
        self.output_directory.as_ref()
    }
}
