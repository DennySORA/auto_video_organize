use anyhow::{Context, Result};
use log::{debug, error};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 縮圖尺寸設定
pub const THUMBNAIL_WIDTH: u32 = 320;
pub const THUMBNAIL_HEIGHT: u32 = 180;

/// 兩段式 seek 的前置緩衝時間（秒）
const SEEK_MARGIN: f64 = 2.0;

/// 縮圖擷取任務
#[derive(Debug, Clone)]
pub struct ThumbnailTask {
    pub video_path: PathBuf,
    pub timestamp: f64,
    pub output_path: PathBuf,
    pub index: usize,
}

/// 縮圖擷取結果
#[derive(Debug)]
pub struct ThumbnailResult {
    pub output_path: PathBuf,
    pub index: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 擷取單一縮圖（使用兩段式 seek 加速）
///
/// 兩段式 seek：
/// 1. `-ss` 在 `-i` 前：快速跳轉到最近的關鍵幀
/// 2. `-ss` 在 `-i` 後：精準解碼到目標時間點
#[must_use] 
pub fn extract_thumbnail(task: &ThumbnailTask) -> ThumbnailResult {
    let result = extract_thumbnail_inner(task);

    match result {
        Ok(()) => ThumbnailResult {
            output_path: task.output_path.clone(),
            index: task.index,
            success: true,
            error_message: None,
        },
        Err(e) => ThumbnailResult {
            output_path: task.output_path.clone(),
            index: task.index,
            success: false,
            error_message: Some(e.to_string()),
        },
    }
}

fn extract_thumbnail_inner(task: &ThumbnailTask) -> Result<()> {
    // 計算兩段式 seek 的時間點
    let t0 = (task.timestamp - SEEK_MARGIN).max(0.0);
    let delta = task.timestamp - t0;

    debug!(
        "擷取縮圖 {}: timestamp={:.2}s, seek={:.2}s+{:.2}s",
        task.index, task.timestamp, t0, delta
    );

    // 建立縮放和填充濾鏡（保持 16:9 比例，不足部分填黑）
    let filter = format!(
        "scale={THUMBNAIL_WIDTH}:{THUMBNAIL_HEIGHT}:force_original_aspect_ratio=decrease,pad={THUMBNAIL_WIDTH}:{THUMBNAIL_HEIGHT}:(ow-iw)/2:(oh-ih)/2:black"
    );

    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
    ];

    // 第一個 -ss（在 -i 前）：快速跳轉
    if t0 > 0.0 {
        args.push("-ss".to_string());
        args.push(format!("{t0:.3}"));
    }

    args.push("-i".to_string());
    args.push(task.video_path.to_string_lossy().to_string());

    // 第二個 -ss（在 -i 後）：精準定位
    if delta > 0.0 {
        args.push("-ss".to_string());
        args.push(format!("{delta:.3}"));
    }

    args.extend([
        "-frames:v".to_string(),
        "1".to_string(),
        "-an".to_string(),
        "-sn".to_string(),
        "-dn".to_string(),
        "-threads".to_string(),
        "1".to_string(),
        "-vf".to_string(),
        filter,
        "-q:v".to_string(),
        "2".to_string(),
        "-y".to_string(),
        task.output_path.to_string_lossy().to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .with_context(|| format!("無法執行 ffmpeg 擷取縮圖: {}", task.video_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg 擷取縮圖失敗: {}", stderr.trim());
    }

    // 確認輸出檔案存在
    if !task.output_path.exists() {
        anyhow::bail!("縮圖檔案未建立: {}", task.output_path.display());
    }

    Ok(())
}

/// 平行擷取多個縮圖
///
/// 使用 rayon 進行平行處理，每個 ffmpeg 程序使用單執行緒
/// 以避免 CPU 過度訂閱
pub fn extract_thumbnails_parallel(
    tasks: Vec<ThumbnailTask>,
    shutdown_signal: &Arc<AtomicBool>,
) -> Vec<ThumbnailResult> {
    tasks
        .par_iter()
        .map(|task| {
            if shutdown_signal.load(Ordering::SeqCst) {
                return ThumbnailResult {
                    output_path: task.output_path.clone(),
                    index: task.index,
                    success: false,
                    error_message: Some("操作已取消".to_string()),
                };
            }

            let result = extract_thumbnail(task);

            if let Some(msg) = result.error_message.as_ref().filter(|_| !result.success) {
                error!("縮圖擷取失敗 [{}]: {}", task.index, &msg);
            }

            result
        })
        .collect()
}

/// 建立縮圖任務列表
#[must_use]
pub fn create_thumbnail_tasks(
    video_path: &Path,
    timestamps: &[f64],
    output_dir: &Path,
) -> Vec<ThumbnailTask> {
    timestamps
        .iter()
        .enumerate()
        .map(|(i, &timestamp)| ThumbnailTask {
            video_path: video_path.to_path_buf(),
            timestamp,
            output_path: output_dir.join(format!("thumb_{i:03}.jpg")),
            index: i,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_thumbnail_tasks() {
        let video_path = Path::new("/test/video.mp4");
        let timestamps = vec![1.0, 2.0, 3.0];
        let output_dir = Path::new("/test/output");

        let tasks = create_thumbnail_tasks(video_path, &timestamps, output_dir);

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].index, 0);
        assert!((tasks[0].timestamp - 1.0).abs() < 0.01);
        assert_eq!(
            tasks[0].output_path,
            PathBuf::from("/test/output/thumb_000.jpg")
        );
        assert_eq!(tasks[2].index, 2);
        assert_eq!(
            tasks[2].output_path,
            PathBuf::from("/test/output/thumb_002.jpg")
        );
    }

    #[test]
    fn test_thumbnail_task_clone() {
        let task = ThumbnailTask {
            video_path: PathBuf::from("/test/video.mp4"),
            timestamp: 10.5,
            output_path: PathBuf::from("/test/thumb.jpg"),
            index: 0,
        };

        let cloned = task.clone();
        assert_eq!(cloned.video_path, task.video_path);
        assert!((cloned.timestamp - task.timestamp).abs() < 0.01);
    }
}
