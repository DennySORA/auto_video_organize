use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
}

/// 掃描目錄下所有檔案，不過濾檔案類型，按大小排序（由小到大）
pub fn scan_all_files(directory: &Path) -> Result<Vec<FileInfo>> {
    let mut files: Vec<FileInfo> = WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            Some(FileInfo {
                path: entry.into_path(),
                size: metadata.len(),
            })
        })
        .collect();

    files.sort_by_key(|file| file.size);
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scan_all_files() {
        let temp_dir = TempDir::new().unwrap();

        // 建立測試檔案
        let file1_path = temp_dir.path().join("small.txt");
        let file2_path = temp_dir.path().join("large.txt");

        {
            let mut file1 = File::create(&file1_path).unwrap();
            file1.write_all(b"small").unwrap();
        }
        {
            let mut file2 = File::create(&file2_path).unwrap();
            file2.write_all(b"this is a larger file content").unwrap();
        }

        let files = scan_all_files(temp_dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        // 應該按大小排序，小的在前
        assert!(files[0].size < files[1].size);
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let files = scan_all_files(temp_dir.path()).unwrap();
        assert!(files.is_empty());
    }
}
