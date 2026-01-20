use super::task_scheduler::{EncodingTask, TaskScheduler, TaskStatus};
use crate::config::Config;
use crate::config::save::{add_recent_path, save_settings};
use crate::tools::{scan_video_files, validate_directory_exists};
use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct VideoEncoder {
    config: Config,
    shutdown_signal: Arc<AtomicBool>,
}

impl VideoEncoder {
    pub const fn new(config: Config, shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            config,
            shutdown_signal,
        }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 影片重新編碼 ===").cyan().bold());

        let Some(input_path) = self.prompt_input_path()? else {
            return Ok(()); // ESC pressed
        };
        let directory = PathBuf::from(&input_path);

        validate_directory_exists(&directory)?;

        // 更新路徑歷史並儲存
        {
            let mut settings = self.config.settings.clone();
            add_recent_path(&mut settings, &input_path);
            if let Err(e) = save_settings(&settings) {
                warn!("無法儲存路徑歷史: {e}");
            }
        }

        println!("{}", style("掃描影片檔案中...").dim());
        let video_files = scan_video_files(&directory, &self.config.file_type_table)?;

        if video_files.is_empty() {
            println!("{}", style("找不到任何影片檔案").yellow());
            return Ok(());
        }

        println!(
            "{}",
            style(format!(
                "找到 {} 個影片檔案，依檔案大小排序（由小到大）：",
                video_files.len()
            ))
            .green()
        );

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
        // 顯示轉檔後處理設定
        let post_action = self.config.settings.video_encoder.post_encode_action;
        if post_action != crate::config::PostEncodeAction::None {
            println!("{}", style(format!("轉檔後處理: {post_action}")).dim());
        }

        println!("{}", style("開始編碼任務...").cyan());

        let mut scheduler = TaskScheduler::new(
            video_files,
            &directory,
            Arc::clone(&self.shutdown_signal),
            post_action,
        )?;

        if let Err(e) = scheduler.run() {
            error!("編碼任務執行失敗: {e}");
            return Err(e);
        }

        self.print_summary(scheduler.tasks());

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<Option<String>> {
        let recent_paths = &self.config.settings.recent_paths;

        // 如果沒有歷史路徑，直接輸入
        if recent_paths.is_empty() {
            let path: String = Input::new()
                .with_prompt("請輸入影片資料夾路徑")
                .interact_text()?;
            return Ok(Some(path.trim().to_string()));
        }

        // 建立選項清單：歷史路徑 + 輸入新路徑
        let mut options: Vec<String> = recent_paths
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let exists = Path::new(p).exists();
                let indicator = if exists { "✓" } else { "✗" };
                format!("{} [{}] {}", i + 1, indicator, p)
            })
            .collect();
        options.push("輸入新路徑...".to_string());

        println!("{}", style("(按 ESC 返回主選單)").dim());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇路徑")
            .items(&options)
            .default(0)
            .interact_opt()?;

        match selection {
            None => Ok(None),
            Some(idx) if idx < recent_paths.len() => Ok(Some(recent_paths[idx].clone())),
            Some(_) => {
                let path: String = Input::new()
                    .with_prompt("請輸入影片資料夾路徑")
                    .interact_text()?;
                Ok(Some(path.trim().to_string()))
            }
        }
    }

    fn print_summary(&self, tasks: &[EncodingTask]) {
        let completed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();

        println!();
        println!("{}", style("=== 編碼任務摘要 ===").cyan().bold());
        println!("  總計: {} 個檔案", tasks.len());
        println!("  成功: {} 個", style(completed).green());
        if failed > 0 {
            println!("  失敗: {} 個", style(failed).red());
            println!();
            println!("{}", style("失敗的檔案已移動到 fail 資料夾").yellow());
        }

        info!("編碼任務完成 - 成功: {completed}, 失敗: {failed}");
    }
}
