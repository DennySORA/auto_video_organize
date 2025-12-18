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
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 預覽圖預設輸出子目錄名稱
const CONTACT_SHEET_OUTPUT_DIR: &str = "_contact_sheets";

/// 處理階段數量（A-E 共 5 階段）
const STAGE_COUNT: u64 = 5;

/// 產生唯一 ID（結合時間戳與執行緒 ID）
fn generate_unique_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let thread_id = std::thread::current().id();
    format!("{timestamp:x}_{thread_id:?}")
        .replace("ThreadId(", "")
        .replace(")", "")
}

/// 建立總進度條樣式
fn create_main_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{prefix:.bold.cyan} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
        .unwrap()
        .progress_chars("━━─")
}

/// 建立單一影片進度條樣式
fn create_video_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("  {spinner:.green} {prefix:.bold} [{bar:20.green/dim}] {msg}")
        .unwrap()
        .progress_chars("▓▒░")
}

/// 截斷名稱以適應顯示寬度
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.chars().count() <= max_len {
        format!("{name:<width$}", width = max_len)
    } else {
        let truncated: String = name.chars().take(max_len - 2).collect();
        format!("{truncated}..")
    }
}

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

        // 輸出路徑固定為影片目錄下的子目錄
        let output_dir = input_dir.join(CONTACT_SHEET_OUTPUT_DIR);
        ensure_directory_exists(&output_dir)?;

        println!("預覽圖將輸出至: {}", style(output_dir.display()).cyan());

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
        println!(
            "{}",
            style(format!(
                "開始平行生成預覽圖（使用 {} 個執行緒）...",
                rayon::current_num_threads()
            ))
            .cyan()
        );

        // 平行處理所有影片
        let result = self.process_videos_parallel(&video_files, &output_dir);

        self.print_summary(&result);

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入影片資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
    }

    /// 平行處理所有影片，吃滿 CPU
    fn process_videos_parallel(
        &self,
        videos: &[VideoFileInfo],
        output_dir: &Path,
    ) -> GenerationResult {
        let successful = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);
        let total = videos.len();

        // 建立多重進度條容器
        let multi_progress = MultiProgress::new();

        // 總進度條（放在最上方）
        let main_pb = multi_progress.add(ProgressBar::new(total as u64));
        main_pb.set_style(create_main_progress_style());
        main_pb.set_prefix("總進度");
        main_pb.enable_steady_tick(Duration::from_millis(100));

        // 分隔線
        let separator = multi_progress.add(ProgressBar::new(0));
        separator.set_style(
            ProgressStyle::default_bar()
                .template("─────────────────────────────────────────────────────────")
                .unwrap(),
        );
        separator.tick();

        videos.par_iter().for_each(|video| {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                return;
            }

            let video_name = video.path.file_stem().map_or_else(
                || "unknown".to_string(),
                |s| s.to_string_lossy().to_string(),
            );

            // 檢查輸出檔案是否已存在
            let output_path = output_dir.join(format!("{video_name}_contact_sheet.jpg"));
            if output_path.exists() {
                info!("{video_name}: 預覽圖已存在，跳過");
                skipped.fetch_add(1, Ordering::SeqCst);
                main_pb.inc(1);
                main_pb.set_message(format!("跳過: {video_name}"));
                return;
            }

            // 為此影片建立進度條
            let video_pb = multi_progress.add(ProgressBar::new(STAGE_COUNT));
            video_pb.set_style(create_video_progress_style());
            video_pb.set_prefix(truncate_name(&video_name, 20));
            video_pb.enable_steady_tick(Duration::from_millis(80));

            match self.process_single_video_with_progress(&video.path, &output_path, &video_pb) {
                Ok(()) => {
                    video_pb.set_message("✓ 完成");
                    video_pb.finish();
                    info!("{video_name}: 預覽圖已建立");
                    successful.fetch_add(1, Ordering::SeqCst);
                }
                Err(e) => {
                    video_pb.set_message(format!("✗ {e}"));
                    video_pb.abandon();
                    error!("{video_name}: 處理失敗 - {e}");
                    failed.fetch_add(1, Ordering::SeqCst);
                }
            }

            main_pb.inc(1);
            main_pb.set_message(format!(
                "成功: {} / 失敗: {} / 跳過: {}",
                successful.load(Ordering::SeqCst),
                failed.load(Ordering::SeqCst),
                skipped.load(Ordering::SeqCst)
            ));

            // 移除已完成的影片進度條
            multi_progress.remove(&video_pb);
        });

        main_pb.finish_with_message("處理完成");

        GenerationResult {
            total_videos: total,
            successful: successful.load(Ordering::SeqCst),
            failed: failed.load(Ordering::SeqCst),
            skipped: skipped.load(Ordering::SeqCst),
        }
    }

    fn process_single_video_with_progress(
        &self,
        video_path: &Path,
        output_path: &Path,
        progress: &ProgressBar,
    ) -> Result<()> {
        // 建立暫存目錄（使用唯一 ID 避免平行處理時衝突）
        let video_stem = video_path
            .file_stem()
            .map_or_else(|| "video".to_string(), |s| s.to_string_lossy().to_string());

        let unique_id = generate_unique_id();
        let temp_dir = output_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!(".tmp_{video_stem}_{unique_id}"));

        ensure_directory_exists(&temp_dir)?;

        let result =
            self.process_video_stages_with_progress(video_path, output_path, &temp_dir, progress);

        // 清理暫存目錄
        if temp_dir.exists() && fs::remove_dir_all(&temp_dir).is_err() {
            warn!("無法清理暫存目錄: {}", temp_dir.display());
        }

        result
    }

    fn process_video_stages_with_progress(
        &self,
        video_path: &Path,
        output_path: &Path,
        temp_dir: &Path,
        progress: &ProgressBar,
    ) -> Result<()> {
        let video_name = video_path.file_name().map_or_else(
            || "unknown".to_string(),
            |s| s.to_string_lossy().to_string(),
        );

        // Stage A: 取得影片資訊
        progress.set_message("A: 讀取資訊");
        debug!("{video_name}: 讀取影片資訊...");
        let video_info = get_video_info(video_path)
            .with_context(|| format!("無法讀取影片資訊: {}", video_path.display()))?;
        debug!(
            "{video_name}: {:.1}s, {}x{}",
            video_info.duration_seconds, video_info.width, video_info.height
        );
        progress.inc(1);

        // 檢查影片是否太短
        if video_info.duration_seconds < 1.0 {
            anyhow::bail!("影片太短（< 1 秒）");
        }

        // Stage B: 場景變換偵測
        progress.set_message("B: 偵測場景");
        debug!("{video_name}: 偵測場景變換...");
        let scenes =
            detect_scenes(video_path, &video_info, None).with_context(|| "場景偵測失敗")?;
        debug!("{video_name}: 找到 {} 個場景變換點", scenes.len());
        progress.inc(1);

        // Stage C: 選取時間點
        progress.set_message("C: 選取時間點");
        debug!("{video_name}: 選取截圖時間點...");
        let timestamps = select_timestamps(
            video_info.duration_seconds,
            &scenes,
            DEFAULT_THUMBNAIL_COUNT,
        );
        debug!("{video_name}: 選取 {} 個時間點", timestamps.len());
        progress.inc(1);

        if timestamps.len() < DEFAULT_THUMBNAIL_COUNT {
            anyhow::bail!(
                "無法選取足夠的時間點: 需要 {}，只有 {}",
                DEFAULT_THUMBNAIL_COUNT,
                timestamps.len()
            );
        }

        // Stage D: 擷取縮圖
        progress.set_message("D: 擷取縮圖");
        debug!("{video_name}: 擷取縮圖...");
        let tasks = create_thumbnail_tasks(video_path, &timestamps, temp_dir);
        let results = extract_thumbnails_parallel(tasks, &self.shutdown_signal);

        let success_count = results.iter().filter(|r| r.success).count();
        let failed_count = results.len() - success_count;
        debug!("{video_name}: 縮圖擷取完成 - 成功 {success_count}, 失敗 {failed_count}");
        progress.inc(1);

        if success_count < DEFAULT_THUMBNAIL_COUNT {
            anyhow::bail!(
                "縮圖擷取失敗: 需要 {DEFAULT_THUMBNAIL_COUNT} 張，只有 {success_count} 張成功"
            );
        }

        // Stage E: 合併預覽圖
        progress.set_message("E: 合併圖片");
        debug!("{video_name}: 合併預覽圖...");

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
        progress.inc(1);

        debug!("{video_name}: 預覽圖生成完成");

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
