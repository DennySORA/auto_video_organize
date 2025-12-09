use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub duration_seconds: f64,
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub frame_rate: f64,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    format: Option<FormatInfo>,
    streams: Option<Vec<StreamInfo>>,
}

#[derive(Deserialize)]
struct FormatInfo {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct StreamInfo {
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    duration: Option<String>,
}

/// 使用 ffprobe 取得影片資訊
pub fn get_video_info(path: &Path) -> Result<VideoInfo> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .with_context(|| format!("無法執行 ffprobe: {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe 執行失敗: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe: FfprobeOutput =
        serde_json::from_str(&stdout).with_context(|| "無法解析 ffprobe 輸出")?;

    // 找到視訊串流
    let video_stream = probe
        .streams
        .as_ref()
        .and_then(|streams| {
            streams
                .iter()
                .find(|s| s.codec_type.as_deref() == Some("video"))
        })
        .ok_or_else(|| anyhow::anyhow!("找不到視訊串流: {}", path.display()))?;

    // 取得寬度和高度
    let width = video_stream
        .width
        .ok_or_else(|| anyhow::anyhow!("無法取得影片寬度"))?;
    let height = video_stream
        .height
        .ok_or_else(|| anyhow::anyhow!("無法取得影片高度"))?;

    // 取得影片長度（優先從 format，其次從 stream）
    let duration_seconds = probe
        .format
        .as_ref()
        .and_then(|f| f.duration.as_ref())
        .or(video_stream.duration.as_ref())
        .and_then(|d| d.parse::<f64>().ok())
        .ok_or_else(|| anyhow::anyhow!("無法取得影片長度"))?;

    // 解析幀率（格式可能是 "30/1" 或 "30000/1001"）
    let frame_rate = video_stream
        .r_frame_rate
        .as_ref()
        .and_then(|r| parse_frame_rate(r))
        .unwrap_or(30.0);

    Ok(VideoInfo {
        duration_seconds,
        width,
        height,
        frame_rate,
    })
}

/// 解析幀率字串（例如 "30/1" 或 "30000/1001"）
fn parse_frame_rate(rate: &str) -> Option<f64> {
    if let Some((num_str, den_str)) = rate.split_once('/') {
        let num: f64 = num_str.parse().ok()?;
        let den: f64 = den_str.parse().ok()?;
        if den > 0.0 {
            return Some(num / den);
        }
    }
    rate.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame_rate_fraction() {
        assert!((parse_frame_rate("30/1").unwrap() - 30.0).abs() < 0.01);
        assert!((parse_frame_rate("30000/1001").unwrap() - 29.97).abs() < 0.01);
        assert!((parse_frame_rate("24/1").unwrap() - 24.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_frame_rate_decimal() {
        assert!((parse_frame_rate("29.97").unwrap() - 29.97).abs() < 0.01);
        assert!((parse_frame_rate("60").unwrap() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_frame_rate_invalid() {
        assert!(parse_frame_rate("invalid").is_none());
        assert!(parse_frame_rate("30/0").is_none());
    }
}
