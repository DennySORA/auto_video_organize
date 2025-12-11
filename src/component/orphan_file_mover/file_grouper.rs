//! 檔案分組器
//!
//! 掃描資料夾，將檔案依同名分組，並識別孤立檔案

use crate::tools::{ensure_directory_exists, validate_directory_exists};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// 預設的孤立檔案目標資料夾名稱
pub const DEFAULT_ORPHAN_FOLDER: &str = "orphan_files";

/// 孤立檔案移動結果
#[derive(Debug, Default)]
pub struct OrphanMoveResult {
    /// 掃描的總檔案數
    pub total_files: usize,
    /// 有對應檔案的數量（保留）
    pub files_with_pairs: usize,
    /// 孤立檔案數量（已移動）
    pub orphan_files_moved: usize,
    /// 跳過的檔案數（已存在於目標目錄）
    pub skipped: usize,
    /// 錯誤數量
    pub errors: usize,
}

/// 檔案分組資訊
#[derive(Debug, Clone)]
pub struct FileGroup {
    /// 檔案名稱（不含副檔名）
    pub stem: String,
    /// 屬於此群組的檔案路徑列表
    pub files: Vec<PathBuf>,
}

impl FileGroup {
    /// 檢查是否為孤立群組（只有一個檔案）
    #[must_use]
    pub const fn is_orphan(&self) -> bool {
        self.files.len() == 1
    }

    /// 取得孤立檔案（如果是孤立群組）
    #[must_use]
    pub fn orphan_file(&self) -> Option<&PathBuf> {
        if self.is_orphan() {
            self.files.first()
        } else {
            None
        }
    }
}

/// 檔案分組器
pub struct FileGrouper {
    shutdown_signal: Arc<AtomicBool>,
    /// 目標資料夾名稱
    orphan_folder_name: String,
}

impl FileGrouper {
    /// 建立新的檔案分組器
    #[must_use]
    pub fn new(shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            shutdown_signal,
            orphan_folder_name: DEFAULT_ORPHAN_FOLDER.to_string(),
        }
    }

    /// 設定目標資料夾名稱
    #[must_use]
    pub fn with_orphan_folder_name(mut self, name: impl Into<String>) -> Self {
        self.orphan_folder_name = name.into();
        self
    }

    /// 掃描並分組檔案
    pub fn scan_and_group(&self, directory: &Path) -> Result<Vec<FileGroup>> {
        validate_directory_exists(directory)?;

        info!("開始掃描目錄: {}", directory.display());

        let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // 讀取目錄中的檔案
        let entries = fs::read_dir(directory)
            .with_context(|| format!("無法讀取目錄: {}", directory.display()))?;

        for entry in entries {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                info!("收到中斷訊號，停止掃描");
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("讀取目錄項目失敗: {e}");
                    continue;
                }
            };

            let path = entry.path();

            // 跳過目錄
            if path.is_dir() {
                continue;
            }

            // 跳過隱藏檔案
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
            {
                continue;
            }

            // 取得檔案名稱（不含副檔名）
            let stem = match path.file_stem() {
                Some(s) => s.to_string_lossy().to_string(),
                None => continue,
            };

            // 跳過空名稱
            if stem.is_empty() {
                continue;
            }

            groups.entry(stem).or_default().push(path);
        }

        // 轉換為 FileGroup 向量
        let result: Vec<FileGroup> = groups
            .into_iter()
            .map(|(stem, files)| FileGroup { stem, files })
            .collect();

        info!("掃描完成，找到 {} 個檔案群組", result.len());

        Ok(result)
    }

    /// 移動孤立檔案到目標資料夾
    pub fn move_orphan_files(
        &self,
        groups: &[FileGroup],
        base_dir: &Path,
    ) -> Result<OrphanMoveResult> {
        let orphan_dir = base_dir.join(&self.orphan_folder_name);
        ensure_directory_exists(&orphan_dir)?;

        let moved_count = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);
        let skipped_count = AtomicUsize::new(0);

        let mut total_files = 0;
        let mut files_with_pairs = 0;

        for group in groups {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                info!("收到中斷訊號，停止移動");
                break;
            }

            total_files += group.files.len();

            if group.is_orphan() {
                // 孤立檔案，需要移動
                if let Some(orphan_path) = group.orphan_file() {
                    let file_name = orphan_path.file_name().unwrap_or_default();
                    let target_path = orphan_dir.join(file_name);

                    // 檢查目標是否已存在
                    if target_path.exists() {
                        debug!("跳過已存在的檔案: {}", target_path.display());
                        skipped_count.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }

                    // 移動檔案
                    match fs::rename(orphan_path, &target_path) {
                        Ok(()) => {
                            debug!(
                                "移動孤立檔案: {} -> {}",
                                orphan_path.display(),
                                target_path.display()
                            );
                            moved_count.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(e) => {
                            // 嘗試複製後刪除（跨檔案系統）
                            if let Err(copy_err) = self.copy_and_delete(orphan_path, &target_path) {
                                warn!(
                                    "移動檔案失敗 {}: {} (原始錯誤: {})",
                                    orphan_path.display(),
                                    copy_err,
                                    e
                                );
                                error_count.fetch_add(1, Ordering::SeqCst);
                            } else {
                                moved_count.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                }
            } else {
                // 有對應檔案，保留
                files_with_pairs += group.files.len();
            }
        }

        Ok(OrphanMoveResult {
            total_files,
            files_with_pairs,
            orphan_files_moved: moved_count.load(Ordering::SeqCst),
            skipped: skipped_count.load(Ordering::SeqCst),
            errors: error_count.load(Ordering::SeqCst),
        })
    }

    /// 複製檔案後刪除原檔案
    fn copy_and_delete(&self, source: &Path, target: &Path) -> Result<()> {
        fs::copy(source, target).with_context(|| {
            format!("複製檔案失敗: {} -> {}", source.display(), target.display())
        })?;

        fs::remove_file(source).with_context(|| format!("刪除原檔案失敗: {}", source.display()))?;

        Ok(())
    }

    /// 取得孤立檔案列表（不執行移動）
    #[must_use]
    pub fn get_orphan_files(groups: &[FileGroup]) -> Vec<&PathBuf> {
        groups.iter().filter_map(|g| g.orphan_file()).collect()
    }

    /// 取得有對應檔案的群組列表
    #[must_use]
    pub fn get_paired_groups(groups: &[FileGroup]) -> Vec<&FileGroup> {
        groups.iter().filter(|g| !g.is_orphan()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_grouper() -> FileGrouper {
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        FileGrouper::new(shutdown_signal)
    }

    #[test]
    fn test_file_group_is_orphan() {
        let orphan = FileGroup {
            stem: "test".to_string(),
            files: vec![PathBuf::from("/test/test.mp4")],
        };
        assert!(orphan.is_orphan());

        let paired = FileGroup {
            stem: "video".to_string(),
            files: vec![
                PathBuf::from("/test/video.mp4"),
                PathBuf::from("/test/video.jpg"),
            ],
        };
        assert!(!paired.is_orphan());
    }

    #[test]
    fn test_scan_and_group() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // 建立測試檔案
        // 有對應的檔案組
        fs::write(base_path.join("video1.mp4"), "video content").unwrap();
        fs::write(base_path.join("video1.jpg"), "thumbnail").unwrap();
        // 孤立檔案
        fs::write(base_path.join("orphan.txt"), "alone").unwrap();
        // 三個同名不同副檔名
        fs::write(base_path.join("multi.mp4"), "video").unwrap();
        fs::write(base_path.join("multi.jpg"), "image").unwrap();
        fs::write(base_path.join("multi.srt"), "subtitle").unwrap();

        let grouper = create_test_grouper();
        let groups = grouper.scan_and_group(base_path).unwrap();

        assert_eq!(groups.len(), 3); // video1, orphan, multi

        let orphan_files = FileGrouper::get_orphan_files(&groups);
        assert_eq!(orphan_files.len(), 1);

        let paired_groups = FileGrouper::get_paired_groups(&groups);
        assert_eq!(paired_groups.len(), 2); // video1 和 multi
    }

    #[test]
    fn test_move_orphan_files() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // 建立測試檔案
        fs::write(base_path.join("paired.mp4"), "video").unwrap();
        fs::write(base_path.join("paired.jpg"), "thumbnail").unwrap();
        fs::write(base_path.join("orphan1.txt"), "alone1").unwrap();
        fs::write(base_path.join("orphan2.doc"), "alone2").unwrap();

        let grouper = create_test_grouper();
        let groups = grouper.scan_and_group(base_path).unwrap();
        let result = grouper.move_orphan_files(&groups, base_path).unwrap();

        assert_eq!(result.total_files, 4);
        assert_eq!(result.files_with_pairs, 2); // paired.mp4 和 paired.jpg
        assert_eq!(result.orphan_files_moved, 2); // orphan1.txt 和 orphan2.doc
        assert_eq!(result.errors, 0);

        // 驗證檔案位置
        assert!(base_path.join("paired.mp4").exists());
        assert!(base_path.join("paired.jpg").exists());
        assert!(!base_path.join("orphan1.txt").exists());
        assert!(!base_path.join("orphan2.doc").exists());
        assert!(base_path.join("orphan_files/orphan1.txt").exists());
        assert!(base_path.join("orphan_files/orphan2.doc").exists());
    }

    #[test]
    fn test_skip_hidden_files() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // 建立測試檔案
        fs::write(base_path.join("normal.txt"), "normal").unwrap();
        fs::write(base_path.join(".hidden"), "hidden").unwrap();
        fs::write(base_path.join(".DS_Store"), "macos").unwrap();

        let grouper = create_test_grouper();
        let groups = grouper.scan_and_group(base_path).unwrap();

        // 只應該有一個群組（normal）
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].stem, "normal");
    }

    #[test]
    fn test_custom_orphan_folder_name() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::write(base_path.join("orphan.txt"), "alone").unwrap();

        let grouper = create_test_grouper().with_orphan_folder_name("moved_files");
        let groups = grouper.scan_and_group(base_path).unwrap();
        let result = grouper.move_orphan_files(&groups, base_path).unwrap();

        assert_eq!(result.orphan_files_moved, 1);
        assert!(base_path.join("moved_files/orphan.txt").exists());
    }
}
