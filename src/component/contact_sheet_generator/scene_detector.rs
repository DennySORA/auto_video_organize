use crate::tools::VideoInfo;
use anyhow::{Context, Result};
use log::debug;
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// 場景變換點資訊
#[derive(Debug, Clone)]
pub struct SceneChange {
    pub timestamp: f64,
    #[allow(dead_code)]
    pub score: f64,
}

/// 場景偵測設定
pub struct SceneDetectorConfig {
    /// 場景變換閾值 (0-100)，越低越敏感
    pub threshold: f64,
    /// 分析用的 FPS，越低越快但可能漏掉短鏡頭
    pub analyze_fps: f64,
    /// 縮放到的寬度（加速分析）
    pub scale_width: u32,
}

impl Default for SceneDetectorConfig {
    fn default() -> Self {
        Self {
            threshold: 12.0,
            analyze_fps: 2.0,
            scale_width: 320,
        }
    }
}

impl SceneDetectorConfig {
    /// 根據影片長度自動調整參數
    #[must_use]
    pub fn auto_adjust(video_info: &VideoInfo) -> Self {
        let duration = video_info.duration_seconds;

        // 根據影片長度調整 FPS
        let analyze_fps = if duration > 7200.0 {
            // > 2 小時
            0.5
        } else if duration > 3600.0 {
            // > 1 小時
            1.0
        } else {
            2.0
        };

        Self {
            threshold: 12.0,
            analyze_fps,
            scale_width: 320,
        }
    }
}

/// 使用 ffmpeg scdet 濾鏡偵測場景變換
pub fn detect_scenes(
    path: &Path,
    video_info: &VideoInfo,
    config: Option<SceneDetectorConfig>,
) -> Result<Vec<SceneChange>> {
    let config = config.unwrap_or_else(|| SceneDetectorConfig::auto_adjust(video_info));

    debug!(
        "場景偵測設定: threshold={}, analyze_fps={}, scale_width={}",
        config.threshold, config.analyze_fps, config.scale_width
    );

    // 建立 ffmpeg 命令
    // 使用 scdet 濾鏡，輸出場景變換資訊到 stderr
    let filter = format!(
        "scale={}:-1,fps={},scdet=s=1:t={}",
        config.scale_width, config.analyze_fps, config.threshold
    );

    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-i"])
        .arg(path)
        .args([
            "-an", "-sn", "-dn", "-threads", "1", "-vf", &filter, "-f", "null", "-",
        ])
        .output()
        .with_context(|| format!("無法執行 ffmpeg 場景偵測: {}", path.display()))?;

    // scdet 輸出在 stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 解析 scdet 輸出
    // 格式: [Parsed_scdet_N @ 0x...] t:NN.NNNN pts_time:NN.NNNN
    parse_scdet_output(&stderr, video_info.duration_seconds)
}

/// 解析 ffmpeg scdet 輸出
fn parse_scdet_output(output: &str, duration: f64) -> Result<Vec<SceneChange>> {
    let mut scenes = Vec::new();

    // 匹配 scdet 輸出格式
    // 例如: [Parsed_scdet_2 @ 0x7f9...] lavfi.scd.time=12.345
    // 或: [scdet @ 0x...] t:12.345 pts_time:12.345
    let time_regex = Regex::new(r"t:([0-9.]+)")?;
    let scd_time_regex = Regex::new(r"lavfi\.scd\.time=([0-9.]+)")?;

    for line in output.lines() {
        // 嘗試匹配 t: 格式或 lavfi.scd.time 格式
        let timestamp = time_regex
            .captures(line)
            .or_else(|| scd_time_regex.captures(line))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .filter(|&t| t > 0.0 && t < duration);

        if let Some(timestamp) = timestamp {
            scenes.push(SceneChange {
                timestamp,
                score: 1.0, // scdet 不提供分數，預設為 1.0
            });
        }
    }

    // 去重並排序
    scenes.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    scenes.dedup_by(|a, b| (a.timestamp - b.timestamp).abs() < 0.1);

    debug!("偵測到 {} 個場景變換點", scenes.len());

    Ok(scenes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scdet_output_t_format() {
        let output = r"
[Parsed_scdet_2 @ 0x7f9b8c] t:12.345 pts_time:12.345
[Parsed_scdet_2 @ 0x7f9b8c] t:25.678 pts_time:25.678
";
        let scenes = parse_scdet_output(output, 100.0).unwrap();
        assert_eq!(scenes.len(), 2);
        assert!((scenes[0].timestamp - 12.345).abs() < 0.001);
        assert!((scenes[1].timestamp - 25.678).abs() < 0.001);
    }

    #[test]
    fn test_parse_scdet_output_scd_time_format() {
        let output = r"
frame:123 pts:12345 pts_time:12.345
lavfi.scd.time=12.345
frame:456 pts:25678 pts_time:25.678
lavfi.scd.time=25.678
";
        let scenes = parse_scdet_output(output, 100.0).unwrap();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn test_parse_scdet_output_filters_out_of_range() {
        let output = r"
[scdet] t:0.0 pts_time:0.0
[scdet] t:50.0 pts_time:50.0
[scdet] t:150.0 pts_time:150.0
";
        let scenes = parse_scdet_output(output, 100.0).unwrap();
        assert_eq!(scenes.len(), 1);
        assert!((scenes[0].timestamp - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_config_auto_adjust() {
        let short_video = VideoInfo {
            duration_seconds: 600.0,
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
        };
        let config = SceneDetectorConfig::auto_adjust(&short_video);
        assert!((config.analyze_fps - 2.0).abs() < 0.01);

        let long_video = VideoInfo {
            duration_seconds: 7500.0,
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
        };
        let config = SceneDetectorConfig::auto_adjust(&long_video);
        assert!((config.analyze_fps - 0.5).abs() < 0.01);
    }
}
