//! 影片排序模組
//!
//! 負責取得影片時長並依時長排序

use crate::tools::{VideoFileInfo, get_video_info};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// 包含時長資訊的影片結構
#[derive(Debug, Clone)]
pub struct VideoWithDuration {
    /// 影片路徑
    pub path: PathBuf,
    /// 影片時長（秒）
    pub duration_seconds: f64,
    /// 檔案大小（位元組）
    pub size: u64,
}

/// 影片排序器
pub struct VideoSorter;

impl Default for VideoSorter {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoSorter {
    pub const fn new() -> Self {
        Self
    }

    /// 取得影片時長並依時長排序（短到長）
    ///
    /// # Arguments
    /// * `videos` - 影片檔案列表
    /// * `shutdown_signal` - 中斷信號
    ///
    /// # Returns
    /// 依時長排序的影片列表（含時長資訊），以及處理失敗的影片數量
    pub fn sort_by_duration(
        &self,
        videos: Vec<VideoFileInfo>,
        shutdown_signal: &AtomicBool,
    ) -> Result<(Vec<VideoWithDuration>, usize)> {
        let progress_bar = ProgressBar::new(videos.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );
        progress_bar.set_message("取得影片時長中...");

        let results: Mutex<Vec<VideoWithDuration>> = Mutex::new(Vec::with_capacity(videos.len()));
        let failed_count: Mutex<usize> = Mutex::new(0);

        videos.par_iter().for_each(|video| {
            if shutdown_signal.load(Ordering::SeqCst) {
                return;
            }

            match get_video_info(&video.path) {
                Ok(info) => {
                    let video_with_duration = VideoWithDuration {
                        path: video.path.clone(),
                        duration_seconds: info.duration_seconds,
                        size: video.size,
                    };
                    results.lock().unwrap().push(video_with_duration);
                }
                Err(_) => {
                    *failed_count.lock().unwrap() += 1;
                }
            }

            progress_bar.inc(1);
        });

        progress_bar.finish_with_message("完成");

        let mut sorted_videos = results.into_inner().unwrap();
        let failed = *failed_count.lock().unwrap();

        sorted_videos.sort_by(|a, b| {
            a.duration_seconds
                .partial_cmp(&b.duration_seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok((sorted_videos, failed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_with_duration_sorting() {
        let mut videos = [
            VideoWithDuration {
                path: PathBuf::from("/a.mp4"),
                duration_seconds: 120.0,
                size: 1000,
            },
            VideoWithDuration {
                path: PathBuf::from("/b.mp4"),
                duration_seconds: 60.0,
                size: 500,
            },
            VideoWithDuration {
                path: PathBuf::from("/c.mp4"),
                duration_seconds: 180.0,
                size: 2000,
            },
        ];

        videos.sort_by(|a, b| {
            a.duration_seconds
                .partial_cmp(&b.duration_seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(videos[0].duration_seconds, 60.0);
        assert_eq!(videos[1].duration_seconds, 120.0);
        assert_eq!(videos[2].duration_seconds, 180.0);
    }

    #[test]
    fn test_video_sorter_new() {
        let sorter = VideoSorter::new();
        assert!(std::mem::size_of_val(&sorter) == 0);
    }
}
