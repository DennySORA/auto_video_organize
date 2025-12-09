use crate::tools::{
    FileInfo, HashTable, calculate_file_hash, ensure_directory_exists, scan_all_files,
};
use anyhow::{Context, Result};
use log::{error, info};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct DuplicationResult {
    pub total_files: usize,
    pub duplicates_found: usize,
    pub duplicates_moved: usize,
    pub new_files_registered: usize,
    pub errors: usize,
}

pub struct DuplicationDetector {
    hash_table: HashTable,
    hash_table_path: PathBuf,
    duplication_directory: PathBuf,
    shutdown_signal: Arc<AtomicBool>,
}

impl DuplicationDetector {
    pub fn new(
        hash_table_path: &Path,
        base_directory: &Path,
        shutdown_signal: Arc<AtomicBool>,
    ) -> Result<Self> {
        let duplication_directory = base_directory.join("duplication_file");
        ensure_directory_exists(&duplication_directory)?;

        let hash_table = HashTable::load_from_file(hash_table_path)?;

        Ok(Self {
            hash_table,
            hash_table_path: hash_table_path.to_path_buf(),
            duplication_directory,
            shutdown_signal,
        })
    }

    pub fn detect_and_move_duplicates(&mut self, directory: &Path) -> Result<DuplicationResult> {
        info!("開始掃描目錄: {}", directory.display());

        let files = scan_all_files(directory)?;
        let total_files = files.len();

        info!("找到 {total_files} 個檔案，開始去重檢查...");

        let duplicates_found = AtomicUsize::new(0);
        let duplicates_moved = AtomicUsize::new(0);
        let new_files_registered = AtomicUsize::new(0);
        let errors = AtomicUsize::new(0);

        let hash_table = Arc::new(Mutex::new(std::mem::take(&mut self.hash_table)));
        let duplication_directory = self.duplication_directory.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        // 使用 rayon 平行處理
        files.par_iter().for_each(|file| {
            if shutdown_signal.load(Ordering::SeqCst) {
                return;
            }

            match self.process_file(file, &hash_table, &duplication_directory) {
                Ok(ProcessResult::Duplicate) => {
                    duplicates_found.fetch_add(1, Ordering::SeqCst);
                    duplicates_moved.fetch_add(1, Ordering::SeqCst);
                }
                Ok(ProcessResult::New) => {
                    new_files_registered.fetch_add(1, Ordering::SeqCst);
                }
                Err(e) => {
                    error!("處理檔案失敗 {}: {}", file.path.display(), e);
                    errors.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        // 取回 hash_table
        self.hash_table = Arc::try_unwrap(hash_table)
            .map_err(|_| anyhow::anyhow!("無法取回 hash table"))?
            .into_inner()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {e}"))?;

        // 儲存更新後的 hash table
        self.hash_table
            .save_to_file(&self.hash_table_path)
            .with_context(|| "無法儲存 hash table")?;

        let result = DuplicationResult {
            total_files,
            duplicates_found: duplicates_found.load(Ordering::SeqCst),
            duplicates_moved: duplicates_moved.load(Ordering::SeqCst),
            new_files_registered: new_files_registered.load(Ordering::SeqCst),
            errors: errors.load(Ordering::SeqCst),
        };

        info!(
            "去重完成 - 總計: {}, 重複: {}, 新增: {}, 錯誤: {}",
            result.total_files, result.duplicates_found, result.new_files_registered, result.errors
        );

        Ok(result)
    }

    fn process_file(
        &self,
        file: &FileInfo,
        hash_table: &Arc<Mutex<HashTable>>,
        duplication_directory: &Path,
    ) -> Result<ProcessResult> {
        let size = file.size;

        // 先檢查是否有相同大小的檔案
        let has_same_size = {
            let table = hash_table
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock failed: {e}"))?;
            table.has_size(size)
        };

        if !has_same_size {
            // 沒有相同大小的檔案，這是新檔案，計算 hash 並加入
            let hash = calculate_file_hash(&file.path)?;
            let mut table = hash_table
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock failed: {e}"))?;
            table.insert(size, hash);
            return Ok(ProcessResult::New);
        }

        // 有相同大小的檔案，計算 hash 來確認是否重複
        let hash = calculate_file_hash(&file.path)?;

        let is_duplicate = {
            let table = hash_table
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock failed: {e}"))?;
            table.contains_hash(size, &hash)
        };

        if is_duplicate {
            // 是重複檔案，移動到 duplication_file 資料夾
            self.move_to_duplication_folder(file, duplication_directory)?;
            Ok(ProcessResult::Duplicate)
        } else {
            // 相同大小但不同 hash，加入到 hash table
            let mut table = hash_table
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock failed: {e}"))?;
            table.insert(size, hash);
            Ok(ProcessResult::New)
        }
    }

    fn move_to_duplication_folder(
        &self,
        file: &FileInfo,
        duplication_directory: &Path,
    ) -> Result<()> {
        let file_name = file
            .path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("無法取得檔案名稱"))?;

        let mut dest_path = duplication_directory.join(file_name);

        // 如果目標已存在，加上編號
        if dest_path.exists() {
            let stem = file
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = file.path.extension().and_then(|s| s.to_str()).unwrap_or("");

            let mut counter = 1;
            loop {
                let new_name = if ext.is_empty() {
                    format!("{stem}_{counter}")
                } else {
                    format!("{stem}_{counter}.{ext}")
                };
                dest_path = duplication_directory.join(&new_name);
                if !dest_path.exists() {
                    break;
                }
                counter += 1;
            }
        }

        fs::rename(&file.path, &dest_path).with_context(|| {
            format!(
                "無法移動重複檔案: {} -> {}",
                file.path.display(),
                dest_path.display()
            )
        })?;

        info!(
            "移動重複檔案: {} -> {}",
            file.path.display(),
            dest_path.display()
        );

        Ok(())
    }
}

enum ProcessResult {
    Duplicate,
    New,
}
