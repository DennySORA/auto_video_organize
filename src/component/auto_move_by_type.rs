use crate::config::{Config, FileCategory};
use crate::tools::{
    CategorizationResult, CategorizedFile, FileCategorizer, validate_directory_exists,
};
use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input};
use log::{info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 自動依類型移動檔案元件
pub struct AutoMoveByType {
    config: Config,
    shutdown_signal: Arc<AtomicBool>,
}

impl AutoMoveByType {
    pub const fn new(config: Config, shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            config,
            shutdown_signal,
        }
    }

    pub fn run(&self) -> Result<()> {
        println!("{}", style("=== 自動依類型整理檔案 ===").cyan().bold());

        // 取得輸入路徑
        let input_path = self.prompt_input_path()?;
        let directory = PathBuf::from(&input_path);

        validate_directory_exists(&directory)?;

        // 建立分類器
        let categorizer = FileCategorizer::new(
            self.config.file_type_table.clone(),
            Arc::clone(&self.shutdown_signal),
        );

        // 掃描並分類
        println!("{}", style("掃描檔案中...").dim());
        let files = categorizer.scan_and_categorize(&directory)?;

        if files.is_empty() {
            println!("{}", style("找不到任何待分類的檔案").yellow());
            return Ok(());
        }

        // 顯示分類摘要
        self.print_category_summary(&files);

        // 確認是否執行
        if !self.confirm_move()? {
            println!("{}", style("操作已取消").yellow());
            return Ok(());
        }

        // 檢查中斷訊號
        if self.shutdown_signal.load(Ordering::SeqCst) {
            warn!("收到中斷訊號，停止處理");
            return Ok(());
        }

        // 移動檔案
        println!("{}", style("移動檔案中...").cyan());
        let result = categorizer.move_files_to_categories(&files, &directory)?;

        self.print_result(&result);

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入要整理的資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
    }

    fn confirm_move(&self) -> Result<bool> {
        let confirm = Confirm::new()
            .with_prompt("確定要移動這些檔案嗎？")
            .default(true)
            .interact()?;
        Ok(confirm)
    }

    fn print_category_summary(&self, files: &[CategorizedFile]) {
        // 統計各分類
        let mut counts: HashMap<FileCategory, (usize, u64)> = HashMap::new();
        for file in files {
            let entry = counts.entry(file.category).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += file.size;
        }

        println!();
        println!(
            "{}",
            style(format!("找到 {} 個檔案，分類如下：", files.len())).green()
        );
        println!();

        // 按檔案數量排序
        let mut sorted_counts: Vec<_> = counts.into_iter().collect();
        sorted_counts.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        for (category, (count, size)) in sorted_counts {
            let size_mb = size as f64 / 1024.0 / 1024.0;
            let folder_name = category.folder_name();
            let display_name = category.display_name();

            println!(
                "  {} {} ({}) - {} 個檔案，{:.2} MB",
                style("→").dim(),
                style(folder_name).cyan(),
                display_name,
                count,
                size_mb
            );
        }

        println!();
    }

    fn print_result(&self, result: &CategorizationResult) {
        println!();
        println!("{}", style("=== 整理結果 ===").cyan().bold());
        println!("  成功移動: {} 個檔案", style(result.files_moved).green());

        if result.skipped > 0 {
            println!("  已跳過: {} 個檔案", style(result.skipped).yellow());
        }

        if result.errors > 0 {
            println!("  失敗: {} 個檔案", style(result.errors).red());
        }

        // 顯示各分類的統計
        if !result.category_counts.is_empty() {
            println!();
            println!("{}", style("分類統計:").dim());

            let mut sorted_counts: Vec<_> = result.category_counts.iter().collect();
            sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

            for (category, count) in sorted_counts {
                println!(
                    "  {} {}: {} 個",
                    style("•").dim(),
                    category.display_name(),
                    count
                );
            }
        }

        info!(
            "檔案整理完成 - 移動: {}, 跳過: {}, 失敗: {}",
            result.files_moved, result.skipped, result.errors
        );
    }
}
