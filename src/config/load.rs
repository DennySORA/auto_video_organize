use crate::config::types::{Config, FileTypeTable, UserSettings};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// 編譯時嵌入的檔案類型設定（不需要外部檔案）
const FILE_TYPE_TABLE_JSON: &str = include_str!("../data/file_type_table.json");

impl Config {
    pub fn new() -> Result<Self> {
        let file_type_table = Self::load_embedded_file_type_table()?;
        let settings = Self::load_settings().unwrap_or_default();

        Ok(Self {
            file_type_table,
            settings,
        })
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

    /// 從編譯時嵌入的 JSON 載入檔案類型表
    fn load_embedded_file_type_table() -> Result<FileTypeTable> {
        serde_json::from_str(FILE_TYPE_TABLE_JSON).context("無法解析嵌入的檔案類型設定")
    }
}
