use crate::config::{FileCategory, FileTypeTable};
use crate::tools::{FileInfo, ensure_directory_exists, scan_all_files};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// 分類結果
#[derive(Debug, Default)]
pub struct CategorizationResult {
    /// 各分類的檔案數量
    pub category_counts: HashMap<FileCategory, usize>,
    /// 成功移動的檔案數
    pub files_moved: usize,
    /// 移動失敗的檔案數
    pub errors: usize,
    /// 跳過的檔案數（已在目標目錄中）
    pub skipped: usize,
}

impl CategorizationResult {
    /// 取得總檔案數
    #[must_use]
    pub fn total_files(&self) -> usize {
        self.files_moved + self.errors + self.skipped
    }
}

/// 已分類的檔案
#[derive(Debug, Clone)]
pub struct CategorizedFile {
    pub path: PathBuf,
    pub category: FileCategory,
    pub size: u64,
}

/// 檔案分類器
pub struct FileCategorizer {
    file_type_table: FileTypeTable,
    shutdown_signal: Arc<AtomicBool>,
    /// 要排除的資料夾名稱
    exclude_folders: Vec<String>,
}

impl FileCategorizer {
    pub fn new(file_type_table: FileTypeTable, shutdown_signal: Arc<AtomicBool>) -> Self {
        // 預設排除的資料夾（分類目標資料夾）
        let mut exclude_folders: Vec<String> = FileCategory::all_categories()
            .iter()
            .map(|c| c.folder_name().to_string())
            .collect();
        exclude_folders.push("other".to_string());

        Self {
            file_type_table,
            shutdown_signal,
            exclude_folders,
        }
    }

    /// 掃描並分類所有檔案
    pub fn scan_and_categorize(&self, directory: &Path) -> Result<Vec<CategorizedFile>> {
        info!("開始掃描目錄: {}", directory.display());

        // 掃描所有檔案
        let files = scan_all_files(directory)?;

        // 過濾掉已在分類資料夾中的檔案
        let filtered_files: Vec<FileInfo> = files
            .into_iter()
            .filter(|f| !self.is_in_excluded_folder(&f.path, directory))
            .collect();

        info!("掃描到 {} 個待分類檔案", filtered_files.len());

        // 平行分類
        let categorized: Vec<CategorizedFile> = filtered_files
            .par_iter()
            .filter_map(|file| {
                if self.shutdown_signal.load(Ordering::SeqCst) {
                    return None;
                }

                let category = self.file_type_table.categorize_file(&file.path);
                Some(CategorizedFile {
                    path: file.path.clone(),
                    category,
                    size: file.size,
                })
            })
            .collect();

        Ok(categorized)
    }

    /// 檢查檔案是否在排除的資料夾中
    fn is_in_excluded_folder(&self, file_path: &Path, base_dir: &Path) -> bool {
        // 取得相對於 base_dir 的路徑
        if let Ok(relative) = file_path.strip_prefix(base_dir) {
            // 檢查第一層資料夾是否在排除列表中
            if let Some(first_component) = relative.components().next() {
                let folder_name = first_component.as_os_str().to_string_lossy().to_lowercase();
                return self.exclude_folders.iter().any(|f| f == &folder_name);
            }
        }
        false
    }

    /// 移動檔案到對應的分類資料夾
    pub fn move_files_to_categories(
        &self,
        files: &[CategorizedFile],
        base_dir: &Path,
    ) -> Result<CategorizationResult> {
        let mut result = CategorizationResult::default();

        // 建立所需的分類資料夾
        let used_categories: Vec<FileCategory> = files.iter().map(|f| f.category).collect();
        for category in &used_categories {
            let category_dir = base_dir.join(category.folder_name());
            ensure_directory_exists(&category_dir)?;
        }

        // 使用原子計數器
        let moved_count = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);
        let skipped_count = AtomicUsize::new(0);

        // 平行移動檔案
        files.par_iter().for_each(|file| {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                return;
            }

            let target_dir = base_dir.join(file.category.folder_name());
            let file_name = file.path.file_name().unwrap_or_default();
            let target_path = target_dir.join(file_name);

            // 檢查目標檔案是否已存在
            if target_path.exists() {
                debug!("跳過已存在的檔案: {}", target_path.display());
                skipped_count.fetch_add(1, Ordering::SeqCst);
                return;
            }

            // 移動檔案
            match fs::rename(&file.path, &target_path) {
                Ok(()) => {
                    debug!(
                        "移動檔案: {} -> {}",
                        file.path.display(),
                        target_path.display()
                    );
                    moved_count.fetch_add(1, Ordering::SeqCst);
                }
                Err(e) => {
                    // 如果 rename 失敗（可能是跨檔案系統），嘗試複製後刪除
                    if let Err(copy_err) = self.copy_and_delete(&file.path, &target_path) {
                        warn!(
                            "移動檔案失敗 {}: {} (原始錯誤: {})",
                            file.path.display(),
                            copy_err,
                            e
                        );
                        error_count.fetch_add(1, Ordering::SeqCst);
                    } else {
                        moved_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        });

        result.files_moved = moved_count.load(Ordering::SeqCst);
        result.errors = error_count.load(Ordering::SeqCst);
        result.skipped = skipped_count.load(Ordering::SeqCst);

        // 統計各分類數量
        for file in files {
            *result.category_counts.entry(file.category).or_insert(0) += 1;
        }

        Ok(result)
    }

    /// 複製檔案後刪除原檔案
    fn copy_and_delete(&self, source: &Path, target: &Path) -> Result<()> {
        fs::copy(source, target).with_context(|| {
            format!("複製檔案失敗: {} -> {}", source.display(), target.display())
        })?;

        fs::remove_file(source).with_context(|| format!("刪除原檔案失敗: {}", source.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;

    fn create_test_categorizer() -> FileCategorizer {
        let config = Config::new().expect("Failed to load config");
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        FileCategorizer::new(config.file_type_table, shutdown_signal)
    }

    #[test]
    fn test_is_in_excluded_folder() {
        let categorizer = create_test_categorizer();
        let base_dir = Path::new("/test");

        // 在 video 資料夾中的檔案應該被排除
        assert!(categorizer.is_in_excluded_folder(Path::new("/test/video/movie.mp4"), base_dir));

        // 在 image 資料夾中的檔案應該被排除
        assert!(categorizer.is_in_excluded_folder(Path::new("/test/image/photo.jpg"), base_dir));

        // 不在排除資料夾中的檔案不應該被排除
        assert!(
            !categorizer.is_in_excluded_folder(Path::new("/test/downloads/file.txt"), base_dir)
        );

        // 根目錄的檔案不應該被排除
        assert!(!categorizer.is_in_excluded_folder(Path::new("/test/file.txt"), base_dir));
    }

    #[test]
    fn test_scan_and_categorize() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // 建立測試檔案
        fs::write(base_path.join("movie.mp4"), "video content").unwrap();
        fs::write(base_path.join("photo.jpg"), "image content").unwrap();
        fs::write(base_path.join("doc.txt"), "text content").unwrap();
        fs::write(base_path.join("unknown.xyz"), "unknown content").unwrap();

        let categorizer = create_test_categorizer();
        let files = categorizer.scan_and_categorize(base_path).unwrap();

        assert_eq!(files.len(), 4);

        // 確認分類正確
        let video_files: Vec<_> = files
            .iter()
            .filter(|f| f.category == FileCategory::Video)
            .collect();
        assert_eq!(video_files.len(), 1);

        let image_files: Vec<_> = files
            .iter()
            .filter(|f| f.category == FileCategory::Image)
            .collect();
        assert_eq!(image_files.len(), 1);
    }

    #[test]
    fn test_move_files_to_categories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // 建立測試檔案
        fs::write(base_path.join("movie.mp4"), "video content").unwrap();
        fs::write(base_path.join("photo.jpg"), "image content").unwrap();

        let categorizer = create_test_categorizer();
        let files = categorizer.scan_and_categorize(base_path).unwrap();

        let result = categorizer
            .move_files_to_categories(&files, base_path)
            .unwrap();

        assert_eq!(result.files_moved, 2);
        assert_eq!(result.errors, 0);

        // 確認檔案已移動
        assert!(base_path.join("video/movie.mp4").exists());
        assert!(base_path.join("image/photo.jpg").exists());

        // 確認原檔案已不存在
        assert!(!base_path.join("movie.mp4").exists());
        assert!(!base_path.join("photo.jpg").exists());
    }
}
