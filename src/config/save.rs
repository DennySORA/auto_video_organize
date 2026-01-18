use crate::config::types::{MAX_RECENT_PATHS, UserSettings};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn save_settings(settings: &UserSettings) -> Result<()> {
    // Save to settings.json in the current working directory
    let path = Path::new("settings.json");
    let content = serde_json::to_string_pretty(settings).context("Failed to serialize settings")?;

    fs::write(path, content)
        .with_context(|| format!("Failed to write settings to {}", path.display()))?;

    Ok(())
}

/// 更新最近使用的路徑
/// 將新路徑加入最前面，去重並限制數量
pub fn add_recent_path(settings: &mut UserSettings, path: &str) {
    // 移除已存在的相同路徑
    settings.recent_paths.retain(|p| p != path);

    // 加入到最前面
    settings.recent_paths.insert(0, path.to_string());

    // 限制數量
    settings.recent_paths.truncate(MAX_RECENT_PATHS);
}
