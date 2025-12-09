use anyhow::{Result, bail};
use std::path::Path;

pub fn validate_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("路徑不存在: {}", path.display());
    }
    if !path.is_dir() {
        bail!("路徑不是資料夾: {}", path.display());
    }
    Ok(())
}

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
