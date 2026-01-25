use crate::config::FileTypeTable;
use crate::tools::get_video_info;
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct VideoFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub duration_ms: Option<u64>,
}

pub fn scan_video_files(
    directory: &Path,
    file_type_table: &FileTypeTable,
) -> Result<Vec<VideoFileInfo>> {
    let mut video_files: Vec<VideoFileInfo> = WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| file_type_table.is_video_file(entry.path()))
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let duration_ms = get_video_info(entry.path())
                .ok()
                .map(|info| (info.duration_seconds * 1000.0).round() as u64);

            Some(VideoFileInfo {
                path: entry.into_path(),
                size: metadata.len(),
                duration_ms,
            })
        })
        .collect();

    video_files.sort_by_key(|file| file.size);
    Ok(video_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_file_info_sorting() {
        let mut files = [
            VideoFileInfo {
                path: PathBuf::from("/a.mp4"),
                size: 1000,
                duration_ms: Some(10_000),
            },
            VideoFileInfo {
                path: PathBuf::from("/b.mp4"),
                size: 500,
                duration_ms: Some(5_000),
            },
            VideoFileInfo {
                path: PathBuf::from("/c.mp4"),
                size: 2000,
                duration_ms: Some(20_000),
            },
        ];
        files.sort_by_key(|f| f.size);
        assert_eq!(files[0].size, 500);
        assert_eq!(files[1].size, 1000);
        assert_eq!(files[2].size, 2000);
    }
}
