use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::thumbnail_extractor::{THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH};

/// 批次擷取結果
#[derive(Debug)]
pub struct BatchExtractionResult {
    pub thumbnail_paths: Vec<PathBuf>,
    pub success_count: usize,
    pub failed_count: usize,
}

/// 批次擷取配置
pub struct BatchExtractorConfig {
    /// 縮圖寬度
    pub width: u32,
    /// 縮圖高度
    pub height: u32,
    /// JPEG 品質 (1-31，數字越小品質越高)
    pub quality: u8,
}

impl Default for BatchExtractorConfig {
    fn default() -> Self {
        Self {
            width: THUMBNAIL_WIDTH,
            height: THUMBNAIL_HEIGHT,
            quality: 2,
        }
    }
}

/// 批次擷取縮圖
///
/// 使用 ffmpeg 的 select 濾鏡一次擷取多張縮圖，
/// 相比逐一擷取可大幅減少 ffmpeg 進程啟動開銷。
///
/// 策略：將時間點分批處理，每批使用單一 ffmpeg 命令
pub fn extract_thumbnails_batch(
    video_path: &Path,
    timestamps: &[f64],
    output_dir: &Path,
    config: &BatchExtractorConfig,
    shutdown_signal: &Arc<AtomicBool>,
) -> Result<BatchExtractionResult> {
    if timestamps.is_empty() {
        return Ok(BatchExtractionResult {
            thumbnail_paths: Vec::new(),
            success_count: 0,
            failed_count: 0,
        });
    }

    debug!(
        "批次擷取 {} 張縮圖: {}",
        timestamps.len(),
        video_path.display()
    );

    // 分批處理（每批最多 18 張，避免 select 表達式過長）
    const BATCH_SIZE: usize = 18;
    let mut all_paths = Vec::with_capacity(timestamps.len());
    let mut total_success = 0;
    let mut total_failed = 0;

    for (batch_index, batch_timestamps) in timestamps.chunks(BATCH_SIZE).enumerate() {
        if shutdown_signal.load(Ordering::SeqCst) {
            warn!("收到中斷信號，停止批次擷取");
            break;
        }

        let batch_start_index = batch_index * BATCH_SIZE;
        let result = extract_batch(
            video_path,
            batch_timestamps,
            output_dir,
            batch_start_index,
            config,
        )?;

        all_paths.extend(result.thumbnail_paths);
        total_success += result.success_count;
        total_failed += result.failed_count;
    }

    info!(
        "批次擷取完成: 成功 {}, 失敗 {}",
        total_success, total_failed
    );

    Ok(BatchExtractionResult {
        thumbnail_paths: all_paths,
        success_count: total_success,
        failed_count: total_failed,
    })
}

/// 擷取單一批次
fn extract_batch(
    video_path: &Path,
    timestamps: &[f64],
    output_dir: &Path,
    start_index: usize,
    config: &BatchExtractorConfig,
) -> Result<BatchExtractionResult> {
    let mut thumbnail_paths = Vec::with_capacity(timestamps.len());
    let mut success_count = 0;
    let mut failed_count = 0;

    // 建立 select 表達式：選取指定時間點附近的幀
    // 使用 between(t, start, end) 確保能捕捉到目標時間
    let select_expr = build_select_expression(timestamps);

    // 建立縮放濾鏡
    let scale_filter = format!(
        "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black",
        config.width, config.height, config.width, config.height
    );

    // 完整的濾鏡鏈
    let filter_complex = format!("{select_expr},{scale_filter}");

    // 輸出路徑模板
    let output_pattern = output_dir.join(format!("thumb_{:03}_%03d.jpg", start_index / 18));

    let args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-i".to_string(),
        video_path.to_string_lossy().to_string(),
        "-vf".to_string(),
        filter_complex,
        "-vsync".to_string(),
        "vfr".to_string(),
        "-q:v".to_string(),
        config.quality.to_string(),
        "-y".to_string(),
        output_pattern.to_string_lossy().to_string(),
    ];

    debug!("執行批次擷取: ffmpeg {}", args.join(" "));

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .with_context(|| "無法執行 ffmpeg 批次擷取")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("批次擷取失敗，改用逐一擷取: {}", stderr.trim());

        // 降級到逐一擷取
        return extract_individually(video_path, timestamps, output_dir, start_index, config);
    }

    // 收集輸出的縮圖檔案
    for (i, &timestamp) in timestamps.iter().enumerate() {
        let thumb_path = output_dir.join(format!("thumb_{:03}.jpg", start_index + i));

        // 嘗試從批次輸出重命名
        let batch_output =
            output_dir.join(format!("thumb_{:03}_{:03}.jpg", start_index / 18, i + 1));

        if batch_output.exists()
            && let Err(e) = std::fs::rename(&batch_output, &thumb_path)
        {
            warn!(
                "無法重命名縮圖 {} -> {}: {}",
                batch_output.display(),
                thumb_path.display(),
                e
            );
            failed_count += 1;
            continue;
        }

        if thumb_path.exists() {
            thumbnail_paths.push(thumb_path);
            success_count += 1;
        } else {
            // 嘗試單獨擷取這張
            match extract_single_thumbnail(video_path, timestamp, &thumb_path, config) {
                Ok(()) => {
                    thumbnail_paths.push(thumb_path);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("縮圖擷取失敗 [{}]: {}", start_index + i, e);
                    failed_count += 1;
                }
            }
        }
    }

    Ok(BatchExtractionResult {
        thumbnail_paths,
        success_count,
        failed_count,
    })
}

/// 建立 select 濾鏡表達式
fn build_select_expression(timestamps: &[f64]) -> String {
    // 使用 between 確保能捕捉到目標時間附近的幀
    // 容差設為 0.1 秒
    let conditions: Vec<String> = timestamps
        .iter()
        .map(|&t| {
            let start = (t - 0.05).max(0.0);
            let end = t + 0.05;
            format!("between(t\\,{start:.3}\\,{end:.3})")
        })
        .collect();

    format!("select='{}'", conditions.join("+"))
}

/// 逐一擷取（降級方案）
fn extract_individually(
    video_path: &Path,
    timestamps: &[f64],
    output_dir: &Path,
    start_index: usize,
    config: &BatchExtractorConfig,
) -> Result<BatchExtractionResult> {
    let mut thumbnail_paths = Vec::with_capacity(timestamps.len());
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, &timestamp) in timestamps.iter().enumerate() {
        let thumb_path = output_dir.join(format!("thumb_{:03}.jpg", start_index + i));

        match extract_single_thumbnail(video_path, timestamp, &thumb_path, config) {
            Ok(()) => {
                thumbnail_paths.push(thumb_path);
                success_count += 1;
            }
            Err(e) => {
                warn!("縮圖擷取失敗 [{}]: {}", start_index + i, e);
                failed_count += 1;

                // 嘗試產生黑色替代圖片
                if let Ok(()) = generate_black_placeholder(&thumb_path, config) {
                    thumbnail_paths.push(thumb_path);
                    success_count += 1;
                    failed_count -= 1;
                }
            }
        }
    }

    Ok(BatchExtractionResult {
        thumbnail_paths,
        success_count,
        failed_count,
    })
}

/// 擷取單一縮圖（使用快速 seek）
fn extract_single_thumbnail(
    video_path: &Path,
    timestamp: f64,
    output_path: &Path,
    config: &BatchExtractorConfig,
) -> Result<()> {
    // 兩段式 seek：先快速跳轉到附近，再精確定位
    let seek_margin = 2.0;
    let t0 = (timestamp - seek_margin).max(0.0);
    let delta = timestamp - t0;

    let scale_filter = format!(
        "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black",
        config.width, config.height, config.width, config.height
    );

    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
    ];

    // 第一個 -ss（快速跳轉）
    if t0 > 0.0 {
        args.push("-ss".to_string());
        args.push(format!("{t0:.3}"));
    }

    args.push("-i".to_string());
    args.push(video_path.to_string_lossy().to_string());

    // 第二個 -ss（精確定位）
    if delta > 0.0 {
        args.push("-ss".to_string());
        args.push(format!("{delta:.3}"));
    }

    args.extend([
        "-frames:v".to_string(),
        "1".to_string(),
        "-vf".to_string(),
        scale_filter,
        "-q:v".to_string(),
        config.quality.to_string(),
        "-y".to_string(),
        output_path.to_string_lossy().to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .with_context(|| "無法執行 ffmpeg 擷取縮圖")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg 擷取失敗: {}", stderr.trim());
    }

    if !output_path.exists() {
        anyhow::bail!("縮圖未建立: {}", output_path.display());
    }

    Ok(())
}

/// 產生黑色替代圖片
fn generate_black_placeholder(output_path: &Path, config: &BatchExtractorConfig) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=black:s={}x{}:d=1", config.width, config.height),
            "-frames:v",
            "1",
            "-q:v",
            &config.quality.to_string(),
            "-y",
            &output_path.to_string_lossy(),
        ])
        .output()
        .with_context(|| "無法產生替代圖片")?;

    if !output.status.success() {
        anyhow::bail!("產生替代圖片失敗");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_select_expression() {
        let timestamps = vec![1.0, 5.0, 10.0];
        let expr = build_select_expression(&timestamps);

        assert!(expr.contains("select="));
        assert!(expr.contains("between"));
    }

    #[test]
    fn test_batch_extractor_config_default() {
        let config = BatchExtractorConfig::default();
        assert_eq!(config.width, THUMBNAIL_WIDTH);
        assert_eq!(config.height, THUMBNAIL_HEIGHT);
        assert_eq!(config.quality, 2);
    }
}
