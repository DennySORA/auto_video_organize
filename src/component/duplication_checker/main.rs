use super::duplication_detector::{DuplicationDetector, DuplicationResult};
use crate::config::Config;
use crate::config::save::{add_recent_path, save_settings};
use crate::tools::validate_directory_exists;
use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use log::{info, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct DuplicationChecker {
    config: Config,
    shutdown_signal: Arc<AtomicBool>,
}

impl DuplicationChecker {
    pub const fn new(config: Config, shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            config,
            shutdown_signal,
        }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 資料分析紀錄與去重 ===").cyan().bold());

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

        println!("{}", style("掃描檔案中...").dim());

        let hash_table_path = self.get_hash_table_path();

        let mut detector = DuplicationDetector::new(
            &hash_table_path,
            &directory,
            Arc::clone(&self.shutdown_signal),
        )?;

        let result = detector.detect_and_move_duplicates(&directory)?;

        self.print_summary(&result);

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<Option<String>> {
        let recent_paths = &self.config.settings.recent_paths;

        // 如果沒有歷史路徑，直接輸入
        if recent_paths.is_empty() {
            let path: String = Input::new()
                .with_prompt("請輸入要檢查的資料夾路徑")
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
                    .with_prompt("請輸入要檢查的資料夾路徑")
                    .interact_text()?;
                Ok(Some(path.trim().to_string()))
            }
        }
    }

    fn get_hash_table_path(&self) -> PathBuf {
        // 存放在程式執行的當前目錄，方便與程式一起移動
        PathBuf::from("hash_table.json")
    }

    fn print_summary(&self, result: &DuplicationResult) {
        println!();
        println!("{}", style("=== 去重任務摘要 ===").cyan().bold());
        println!("  總計掃描: {} 個檔案", result.total_files);
        println!("  發現重複: {} 個", style(result.duplicates_found).yellow());
        println!(
            "  已移動重複: {} 個",
            style(result.duplicates_moved).green()
        );
        println!(
            "  新增紀錄: {} 個",
            style(result.new_files_registered).green()
        );
        if result.errors > 0 {
            println!("  錯誤: {} 個", style(result.errors).red());
        }

        if result.duplicates_moved > 0 {
            println!();
            println!(
                "{}",
                style("重複檔案已移動到 duplication_file 資料夾").yellow()
            );
        }

        info!(
            "去重任務完成 - 總計: {}, 重複: {}, 新增: {}, 錯誤: {}",
            result.total_files, result.duplicates_found, result.new_files_registered, result.errors
        );
    }
}
