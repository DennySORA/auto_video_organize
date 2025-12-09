use crate::tools::thumbnail_extractor::{THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH};
use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use std::process::Command;

/// 預設網格配置：9 欄 x 6 列 = 54 張縮圖
pub const DEFAULT_GRID_COLS: usize = 9;
pub const DEFAULT_GRID_ROWS: usize = 6;
pub const DEFAULT_THUMBNAIL_COUNT: usize = DEFAULT_GRID_COLS * DEFAULT_GRID_ROWS;

/// 使用 ffmpeg xstack 濾鏡合併縮圖為預覽圖
///
/// xstack 濾鏡比 tile 濾鏡更靈活，可以精確控制每張圖的位置
pub fn create_contact_sheet(
    thumbnails: &[impl AsRef<Path>],
    output_path: &Path,
    grid_cols: usize,
    grid_rows: usize,
) -> Result<()> {
    let expected_count = grid_cols * grid_rows;
    if thumbnails.len() < expected_count {
        anyhow::bail!(
            "縮圖數量不足: 需要 {} 張，但只有 {} 張",
            expected_count,
            thumbnails.len()
        );
    }

    debug!(
        "合併 {} 張縮圖為 {}x{} 預覽圖",
        thumbnails.len(),
        grid_cols,
        grid_rows
    );

    // 建立 xstack 佈局字串
    // 格式: 0_0|w0_0|w0+w1_0|...|0_h0|w0_h0|...
    let layout = build_xstack_layout(grid_cols, grid_rows);

    // 建立 ffmpeg 命令參數
    let mut args: Vec<String> = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
    ];

    // 加入所有輸入檔案
    for (i, thumb) in thumbnails.iter().take(expected_count).enumerate() {
        args.push("-i".to_string());
        args.push(thumb.as_ref().to_string_lossy().to_string());
        debug!("輸入 [{}]: {}", i, thumb.as_ref().display());
    }

    // 建立 filter_complex
    let filter = format!("xstack=inputs={expected_count}:layout={layout}");

    args.extend([
        "-filter_complex".to_string(),
        filter,
        "-frames:v".to_string(),
        "1".to_string(),
        "-y".to_string(),
        output_path.to_string_lossy().to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .with_context(|| "無法執行 ffmpeg 合併預覽圖")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg 合併預覽圖失敗: {}", stderr.trim());
    }

    if !output_path.exists() {
        anyhow::bail!("預覽圖未建立: {}", output_path.display());
    }

    debug!("預覽圖已建立: {}", output_path.display());
    Ok(())
}

/// 建立 xstack 佈局字串
///
/// 每個位置格式為 `x_y，使用` | 分隔
/// 例如 9x6 `網格：0_0|320_0|640_0|...|0_180|320_180`|...
fn build_xstack_layout(cols: usize, rows: usize) -> String {
    let mut positions = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        for col in 0..cols {
            let x = col as u32 * THUMBNAIL_WIDTH;
            let y = row as u32 * THUMBNAIL_HEIGHT;
            positions.push(format!("{x}_{y}"));
        }
    }

    positions.join("|")
}

/// 計算預覽圖的最終尺寸
#[cfg(test)]
const fn calculate_contact_sheet_size(grid_cols: usize, grid_rows: usize) -> (u32, u32) {
    let width = grid_cols as u32 * THUMBNAIL_WIDTH;
    let height = grid_rows as u32 * THUMBNAIL_HEIGHT;
    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_xstack_layout_2x2() {
        let layout = build_xstack_layout(2, 2);
        // 320x180 縮圖
        assert_eq!(layout, "0_0|320_0|0_180|320_180");
    }

    #[test]
    fn test_build_xstack_layout_3x2() {
        let layout = build_xstack_layout(3, 2);
        assert_eq!(layout, "0_0|320_0|640_0|0_180|320_180|640_180");
    }

    #[test]
    fn test_calculate_contact_sheet_size() {
        let (width, height) = calculate_contact_sheet_size(9, 6);
        assert_eq!(width, 9 * 320);
        assert_eq!(height, 6 * 180);
    }

    #[test]
    fn test_default_grid_count() {
        assert_eq!(DEFAULT_THUMBNAIL_COUNT, 54);
        assert_eq!(DEFAULT_GRID_COLS * DEFAULT_GRID_ROWS, 54);
    }
}
