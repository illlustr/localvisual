use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub yt_dlp_path: String,
    pub download_dir: String,
    // UI Constants
    pub row_height: f32,
    pub spacing: f32,
    pub margin: f32,
    pub padding: f32,
    pub icon_button_size: f32,
}

impl Default for Config {
    fn default() -> Self {
        let exe_dir = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        Self {
            yt_dlp_path: exe_dir.join("yt-dlp.exe").to_string_lossy().to_string(),
            download_dir: exe_dir.to_string_lossy().to_string(),
            // Default UI values
            row_height: 28.0,
            spacing: 2.0,
            margin: 8.0,
            padding: 8.0,
            icon_button_size: 28.0,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&contents) {
                return config;
            }
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .map(|p| p.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf())
            .unwrap_or_else(|_| PathBuf::from("."));
        
        exe_dir.join(format!("{}.json", env!("APP_NAME_LOWER")))
    }
}
