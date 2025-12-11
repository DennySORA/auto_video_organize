//! 孤立檔案移動元件
//!
//! 掃描資料夾，將沒有對應檔案（同名不同副檔名）的孤立檔案移動到指定目錄

use super::file_grouper::{FileGroup, FileGrouper, OrphanMoveResult};
use crate::tools::validate_directory_exists;
use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input};
use log::{info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 孤立檔案移動元件
pub struct OrphanFileMover {
    shutdown_signal: Arc<AtomicBool>,
}

impl OrphanFileMover {
    pub const fn new(shutdown_signal: Arc<AtomicBool>) -> Self {
        Self { shutdown_signal }
    }

    pub fn run(&self) -> Result<()> {
        println!(
            "{}",
            style("=== 移動孤立檔案（無對應檔案） ===").cyan().bold()
        );

        // 取得輸入路徑
        let input_path = self.prompt_input_path()?;
        let directory = PathBuf::from(&input_path);

        validate_directory_exists(&directory)?;

        // 建立分組器
        let grouper = FileGrouper::new(Arc::clone(&self.shutdown_signal));

        // 掃描並分組
        println!("{}", style("掃描檔案中...").dim());
        let groups = grouper.scan_and_group(&directory)?;

        if groups.is_empty() {
            println!("{}", style("找不到任何檔案").yellow());
            return Ok(());
        }

        // 顯示分組摘要
        self.print_group_summary(&groups);

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

        // 移動孤立檔案
        println!("{}", style("移動孤立檔案中...").cyan());
        let result = grouper.move_orphan_files(&groups, &directory)?;

        self.print_result(&result);

        Ok(())
    }

    fn prompt_input_path(&self) -> Result<String> {
        let path: String = Input::new()
            .with_prompt("請輸入要處理的資料夾路徑")
            .interact_text()?;
        Ok(path.trim().to_string())
    }

    fn confirm_move(&self) -> Result<bool> {
        let confirm = Confirm::new()
            .with_prompt("確定要移動孤立檔案嗎？")
            .default(true)
            .interact()?;
        Ok(confirm)
    }

    fn print_group_summary(&self, groups: &[FileGroup]) {
        let orphan_files = FileGrouper::get_orphan_files(groups);
        let paired_groups = FileGrouper::get_paired_groups(groups);

        let total_files: usize = groups.iter().map(|g| g.files.len()).sum();
        let paired_files: usize = paired_groups.iter().map(|g| g.files.len()).sum();

        println!();
        println!(
            "{}",
            style(format!(
                "掃描到 {} 個檔案，{} 個群組",
                total_files,
                groups.len()
            ))
            .green()
        );
        println!();

        // 顯示有對應檔案的群組
        if !paired_groups.is_empty() {
            println!(
                "{}",
                style(format!(
                    "有對應檔案的群組（保留） - {} 組，{} 個檔案：",
                    paired_groups.len(),
                    paired_files
                ))
                .cyan()
            );

            // 只顯示前 10 個
            let display_count = paired_groups.len().min(10);
            for group in paired_groups.iter().take(display_count) {
                let extensions: Vec<String> = group
                    .files
                    .iter()
                    .filter_map(|p| p.extension())
                    .map(|e| e.to_string_lossy().to_string())
                    .collect();
                println!(
                    "  {} {} ({})",
                    style("✓").green(),
                    group.stem,
                    extensions.join(", ")
                );
            }
            if paired_groups.len() > display_count {
                println!(
                    "  {} ...還有 {} 組",
                    style("⋯").dim(),
                    paired_groups.len() - display_count
                );
            }
            println!();
        }

        // 顯示孤立檔案
        if orphan_files.is_empty() {
            println!("{}", style("沒有發現孤立檔案").green());
            println!();
        } else {
            println!(
                "{}",
                style(format!("孤立檔案（將移動） - {} 個：", orphan_files.len())).yellow()
            );

            // 只顯示前 10 個
            let display_count = orphan_files.len().min(10);
            for file in orphan_files.iter().take(display_count) {
                let file_name = file
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                println!("  {} {}", style("→").yellow(), file_name);
            }
            if orphan_files.len() > display_count {
                println!(
                    "  {} ...還有 {} 個",
                    style("⋯").dim(),
                    orphan_files.len() - display_count
                );
            }
            println!();
        }
    }

    fn print_result(&self, result: &OrphanMoveResult) {
        println!();
        println!("{}", style("=== 處理結果 ===").cyan().bold());
        println!("  總檔案數: {}", result.total_files);
        println!(
            "  有對應檔案（保留）: {} 個",
            style(result.files_with_pairs).green()
        );
        println!(
            "  孤立檔案（已移動）: {} 個",
            style(result.orphan_files_moved).yellow()
        );

        if result.skipped > 0 {
            println!("  已跳過（目標已存在）: {} 個", style(result.skipped).dim());
        }

        if result.errors > 0 {
            println!("  失敗: {} 個", style(result.errors).red());
        }

        info!(
            "孤立檔案處理完成 - 保留: {}, 移動: {}, 跳過: {}, 失敗: {}",
            result.files_with_pairs, result.orphan_files_moved, result.skipped, result.errors
        );
    }
}
