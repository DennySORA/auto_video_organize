use super::duplication_detector::{DuplicationDetector, DuplicationResult};
use crate::tools::validate_directory_exists;
use anyhow::Result;
use console::style;
use dialoguer::Input;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct DuplicationChecker {
    shutdown_signal: Arc<AtomicBool>,
}

impl DuplicationChecker {
    pub const fn new(shutdown_signal: Arc<AtomicBool>) -> Self {
        Self { shutdown_signal }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 資料分析紀錄與去重 ===").cyan().bold());

        let input_path = self.prompt_input_path()?;
        let directory = PathBuf::from(&input_path);

        validate_directory_exists(&directory)?;

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

    fn prompt_input_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入要檢查的資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
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
