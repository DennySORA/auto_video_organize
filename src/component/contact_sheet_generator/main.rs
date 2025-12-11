use super::contact_sheet_merger::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, create_contact_sheet,
};
use super::scene_detector::detect_scenes;
use super::thumbnail_extractor::{create_thumbnail_tasks, extract_thumbnails_parallel};
use super::timestamp_selector::select_timestamps;
use crate::config::Config;
use crate::tools::{
    VideoFileInfo, ensure_directory_exists, get_video_info, scan_video_files,
    validate_directory_exists,
};
use anyhow::{Context, Result};
use console::style;
use dialoguer::Input;
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 預覽圖生成結果
#[derive(Debug)]
pub struct GenerationResult {
    pub total_videos: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// 預覽圖生成器
///
/// 五階段流程：
/// A. 取得影片資訊（ffprobe）
/// B. 場景變換偵測（scdet）
/// C. 選取 54 個代表時間點
/// D. 平行擷取縮圖
/// E. 合併為 9x6 預覽圖
pub struct ContactSheetGenerator {
    config: Config,
    shutdown_signal: Arc<AtomicBool>,
}

impl ContactSheetGenerator {
    pub const fn new(config: Config, shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            config,
            shutdown_signal,
        }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 影片預覽圖生成 ===").cyan().bold());

        // 取得輸入路徑
        let input_path = self.prompt_input_path()?;
        let input_dir = PathBuf::from(&input_path);
        validate_directory_exists(&input_dir)?;

        // 取得輸出路徑
        let output_path = self.prompt_output_path()?;
        let output_dir = PathBuf::from(&output_path);
        ensure_directory_exists(&output_dir)?;

        // 掃描影片檔案
        println!("{}", style("掃描影片檔案中...").dim());
        let video_files = scan_video_files(&input_dir, &self.config.file_type_table)?;

        if video_files.is_empty() {
            println!("{}", style("找不到任何影片檔案").yellow());
            return Ok(());
        }

        println!(
            "{}",
            style(format!(
                "找到 {} 個影片檔案，依檔案大小排序（由小到大）",
                video_files.len()
            ))
            .green()
        );

        // 顯示檔案列表
        for (index, file) in video_files.iter().enumerate() {
            let size_mb = file.size as f64 / 1024.0 / 1024.0;
            println!(
                "  {}. {} ({:.2} MB)",
                index + 1,
                file.path.file_name().unwrap_or_default().to_string_lossy(),
                size_mb
            );
        }

        println!();
        println!("{}", style("開始生成預覽圖...").cyan());

        // 處理每個影片
        let result = self.process_videos(&video_files, &output_dir)?;

        self.print_summary(&result);

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入影片資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
    }

    fn prompt_output_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入預覽圖輸出資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
    }

    fn process_videos(
        &self,
        videos: &[VideoFileInfo],
        output_dir: &Path,
    ) -> Result<GenerationResult> {
        let mut successful = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for (index, video) in videos.iter().enumerate() {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                warn!("收到中斷訊號，停止處理");
                break;
            }

            let video_name = video.path.file_stem().map_or_else(
                || format!("video_{index}"),
                |s| s.to_string_lossy().to_string(),
            );

            println!(
                "\n{} [{}/{}] {}",
                style("處理中").cyan(),
                index + 1,
                videos.len(),
                style(&video_name).bold()
            );

            // 檢查輸出檔案是否已存在
            let output_path = output_dir.join(format!("{video_name}_contact_sheet.jpg"));
            if output_path.exists() {
                println!("  {} 預覽圖已存在，跳過", style("⤳").dim());
                skipped += 1;
                continue;
            }

            match self.process_single_video(&video.path, &output_path) {
                Ok(()) => {
                    println!("  {} 預覽圖已建立", style("✓").green());
                    successful += 1;
                }
                Err(e) => {
                    error!("處理影片失敗 {video_name}: {e}");
                    println!("  {} 處理失敗: {}", style("✗").red(), e);
                    failed += 1;
                }
            }
        }

        Ok(GenerationResult {
            total_videos: videos.len(),
            successful,
            failed,
            skipped,
        })
    }

    fn process_single_video(&self, video_path: &Path, output_path: &Path) -> Result<()> {
        // 建立暫存目錄
        let video_stem = video_path
            .file_stem()
            .map_or_else(|| "video".to_string(), |s| s.to_string_lossy().to_string());

        let temp_dir = output_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!(".tmp_{video_stem}"));

        ensure_directory_exists(&temp_dir)?;

        // 使用 scopeguard 確保清理暫存目錄
        let result = self.process_video_stages(video_path, output_path, &temp_dir);

        // 清理暫存目錄
        if temp_dir.exists() && fs::remove_dir_all(&temp_dir).is_err() {
            warn!("無法清理暫存目錄: {}", temp_dir.display());
        }

        result
    }

    fn process_video_stages(
        &self,
        video_path: &Path,
        output_path: &Path,
        temp_dir: &Path,
    ) -> Result<()> {
        // Stage A: 取得影片資訊
        print!("  {} 讀取影片資訊...", style("A").dim());
        let video_info = get_video_info(video_path)
            .with_context(|| format!("無法讀取影片資訊: {}", video_path.display()))?;
        println!(
            " {:.1}s, {}x{}",
            video_info.duration_seconds, video_info.width, video_info.height
        );

        // 檢查影片是否太短
        if video_info.duration_seconds < 1.0 {
            anyhow::bail!("影片太短（< 1 秒）");
        }

        // Stage B: 場景變換偵測
        print!("  {} 偵測場景變換...", style("B").dim());
        let scenes =
            detect_scenes(video_path, &video_info, None).with_context(|| "場景偵測失敗")?;
        println!(" 找到 {} 個場景變換點", scenes.len());

        // Stage C: 選取時間點
        print!("  {} 選取截圖時間點...", style("C").dim());
        let timestamps = select_timestamps(
            video_info.duration_seconds,
            &scenes,
            DEFAULT_THUMBNAIL_COUNT,
        );
        println!(" 選取 {} 個時間點", timestamps.len());

        if timestamps.len() < DEFAULT_THUMBNAIL_COUNT {
            anyhow::bail!(
                "無法選取足夠的時間點: 需要 {}，只有 {}",
                DEFAULT_THUMBNAIL_COUNT,
                timestamps.len()
            );
        }

        // Stage D: 平行擷取縮圖
        print!("  {} 擷取縮圖...", style("D").dim());
        let tasks = create_thumbnail_tasks(video_path, &timestamps, temp_dir);
        let results = extract_thumbnails_parallel(tasks, &self.shutdown_signal);

        let success_count = results.iter().filter(|r| r.success).count();
        let failed_count = results.len() - success_count;
        println!(" 成功 {success_count}, 失敗 {failed_count}");

        if success_count < DEFAULT_THUMBNAIL_COUNT {
            anyhow::bail!(
                "縮圖擷取失敗: 需要 {DEFAULT_THUMBNAIL_COUNT} 張，只有 {success_count} 張成功"
            );
        }

        // Stage E: 合併預覽圖
        print!("  {} 合併預覽圖...", style("E").dim());

        // 收集成功的縮圖路徑（按索引排序）
        let mut thumbnail_paths: Vec<_> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| (r.index, r.output_path.clone()))
            .collect();
        thumbnail_paths.sort_by_key(|(idx, _)| *idx);
        let thumbnail_paths: Vec<_> = thumbnail_paths.into_iter().map(|(_, p)| p).collect();

        create_contact_sheet(
            &thumbnail_paths,
            output_path,
            DEFAULT_GRID_COLS,
            DEFAULT_GRID_ROWS,
        )
        .with_context(|| "合併預覽圖失敗")?;

        println!(" 完成");

        info!("預覽圖已建立: {}", output_path.display());

        Ok(())
    }

    fn print_summary(&self, result: &GenerationResult) {
        println!();
        println!("{}", style("=== 預覽圖生成摘要 ===").cyan().bold());
        println!("  總計: {} 個影片", result.total_videos);
        println!("  成功: {} 個", style(result.successful).green());

        if result.skipped > 0 {
            println!("  跳過: {} 個", style(result.skipped).yellow());
        }

        if result.failed > 0 {
            println!("  失敗: {} 個", style(result.failed).red());
        }

        info!(
            "預覽圖生成完成 - 成功: {}, 跳過: {}, 失敗: {}",
            result.successful, result.skipped, result.failed
        );
    }
}
