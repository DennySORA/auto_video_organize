use crate::config::types::{Config, FileTypeTable, UserSettings};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn get_data_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/data")
        .leak()
}

impl Config {
    pub fn new() -> Result<Self> {
        let data_dir = get_data_dir();
        let file_type_table_path = data_dir.join("file_type_table.json");
        let file_type_table = Self::load_file_type_table(&file_type_table_path)?;
        
        let settings = Self::load_settings().unwrap_or_default();
        
        Ok(Self { file_type_table, settings })
    }

    fn load_settings() -> Result<UserSettings> {
        let path = Path::new("settings.json");
        if !path.exists() {
            return Ok(UserSettings::default());
        }
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read settings from {}", path.display()))?;
            
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse settings from {}", path.display()))
    }

    fn load_file_type_table(path: &Path) -> Result<FileTypeTable> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("無法讀取檔案類型設定: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("無法解析檔案類型設定: {}", path.display()))
    }
}
