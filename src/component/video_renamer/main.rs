//! 影片重新命名主模組
//!
//! 協調影片掃描、排序和重新命名的整體流程

use super::filename_cleaner::FilenameCleaner;
use super::video_sorter::{VideoSorter, VideoWithDuration};
use crate::config::Config;
use crate::tools::{scan_video_files, validate_directory_exists};
use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

/// 影片重新命名器
pub struct VideoRenamer {
    config: Config,
    shutdown_signal: Arc<AtomicBool>,
    filename_cleaner: FilenameCleaner,
    video_sorter: VideoSorter,
}

/// 重新命名結果統計
#[derive(Debug, Default)]
struct RenameResult {
    success_count: usize,
    skip_count: usize,
    error_count: usize,
}

impl VideoRenamer {
    pub fn new(config: Config, shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            config,
            shutdown_signal,
            filename_cleaner: FilenameCleaner::new(),
            video_sorter: VideoSorter::new(),
        }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 影片依時長排序重新命名 ===").cyan().bold());

        let directory = self.prompt_directory()?;
        validate_directory_exists(&directory)?;

        let start_index = self.prompt_start_index()?;

        println!("{}", style("掃描影片檔案中...").dim());
        let video_files = scan_video_files(&directory, &self.config.file_type_table)?;

        if video_files.is_empty() {
            println!("{}", style("找不到任何影片檔案").yellow());
            return Ok(());
        }

        println!(
            "{}",
            style(format!("找到 {} 個影片檔案", video_files.len())).green()
        );

        println!("{}", style("取得影片時長中...").dim());
        let (sorted_videos, failed_count) = self
            .video_sorter
            .sort_by_duration(video_files, &self.shutdown_signal)?;

        if self.shutdown_signal.load(Ordering::SeqCst) {
            println!("{}", style("操作已取消").yellow());
            return Ok(());
        }

        if failed_count > 0 {
            println!(
                "{}",
                style(format!("警告：{} 個檔案無法取得時長，已跳過", failed_count)).yellow()
            );
        }

        if sorted_videos.is_empty() {
            println!("{}", style("沒有可處理的影片檔案").yellow());
            return Ok(());
        }

        self.display_preview(&sorted_videos, start_index);

        if !self.confirm_rename()? {
            println!("{}", style("操作已取消").yellow());
            return Ok(());
        }

        let result = self.execute_rename(&sorted_videos, start_index)?;
        self.display_summary(&result);

        Ok(())
    }

    fn prompt_directory(&self) -> Result<PathBuf> {
        let path: String = Input::new()
            .with_prompt("請輸入影片資料夾路徑")
            .interact_text()?;
        Ok(PathBuf::from(path.trim()))
    }

    fn prompt_start_index(&self) -> Result<usize> {
        let index: usize = Input::new()
            .with_prompt("請輸入起始編號")
            .default(1)
            .interact_text()?;
        Ok(index)
    }

    fn confirm_rename(&self) -> Result<bool> {
        let confirmed = Confirm::new()
            .with_prompt("確定要重新命名這些檔案嗎？")
            .default(false)
            .interact()?;
        Ok(confirmed)
    }

    fn display_preview(&self, videos: &[VideoWithDuration], start_index: usize) {
        println!();
        println!(
            "{}",
            style("預覽重新命名結果（依時長排序，短到長）：").cyan()
        );
        println!();

        for (i, video) in videos.iter().enumerate() {
            let current_index = start_index + i;
            let current_name = video.path.file_name().unwrap_or_default().to_string_lossy();
            let cleaned = self.filename_cleaner.clean(&current_name);
            let preview_uuid = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
            let new_name =
                self.filename_cleaner
                    .format_new_filename(current_index, &cleaned, preview_uuid);

            let duration_str = format_duration(video.duration_seconds);

            println!(
                "  {} ({}):",
                style(format!("[{}]", current_index)).dim(),
                style(&duration_str).cyan()
            );
            println!("    {} {}", style("舊:").dim(), current_name);
            println!("    {} {}", style("新:").dim(), new_name);
            println!();
        }
    }

    fn execute_rename(
        &self,
        videos: &[VideoWithDuration],
        start_index: usize,
    ) -> Result<RenameResult> {
        let mut result = RenameResult::default();

        let progress_bar = ProgressBar::new(videos.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );
        progress_bar.set_message("重新命名中...");

        for (i, video) in videos.iter().enumerate() {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                progress_bar.abandon_with_message("操作已中斷");
                break;
            }

            let current_index = start_index + i;
            let current_name = video.path.file_name().unwrap_or_default().to_string_lossy();
            let cleaned = self.filename_cleaner.clean(&current_name);
            let new_uuid = Uuid::new_v4().to_string();
            let new_name =
                self.filename_cleaner
                    .format_new_filename(current_index, &cleaned, &new_uuid);

            let new_path = video.path.parent().unwrap_or(&video.path).join(&new_name);

            if new_path.exists() {
                result.skip_count += 1;
                progress_bar.inc(1);
                continue;
            }

            match fs::rename(&video.path, &new_path) {
                Ok(()) => {
                    result.success_count += 1;
                }
                Err(_) => {
                    result.error_count += 1;
                }
            }

            progress_bar.inc(1);
        }

        progress_bar.finish_with_message("完成");

        Ok(result)
    }

    fn display_summary(&self, result: &RenameResult) {
        println!();
        println!("{}", style("=== 重新命名結果 ===").cyan().bold());
        println!("  成功: {} 個", style(result.success_count).green());
        if result.skip_count > 0 {
            println!("  跳過: {} 個", style(result.skip_count).yellow());
        }
        if result.error_count > 0 {
            println!("  失敗: {} 個", style(result.error_count).red());
        }
    }
}

/// 格式化時長為人類可讀格式
fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(45.0), "00:45");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(125.0), "02:05");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3661.0), "01:01:01");
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0.0), "00:00");
    }
}
