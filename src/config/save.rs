use crate::config::types::UserSettings;
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
