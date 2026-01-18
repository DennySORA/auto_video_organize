//! 均勻時間點選取器
//!
//! 不需要場景偵測，直接根據影片長度均勻分布取樣點。
//! 這是「快速模式」的核心，可跳過耗時的場景分析。

/// 選取均勻分布的時間點
///
/// 策略：在每個時段的中間位置取樣
/// 公式：timestamp[i] = duration * (i + 0.5) / count
///
/// 這樣可以：
/// 1. 避開影片開頭和結尾（通常是片頭/片尾）
/// 2. 確保取樣點均勻分布
/// 3. 在每個時段的中間取樣，避開邊界
#[must_use]
pub fn select_uniform_timestamps(duration: f64, count: usize) -> Vec<f64> {
    if count == 0 || duration <= 0.0 {
        return Vec::new();
    }

    // 預留前後各 2% 的邊界
    let margin_ratio = 0.02;
    let effective_start = duration * margin_ratio;
    let effective_end = duration * (1.0 - margin_ratio);
    let effective_duration = effective_end - effective_start;

    (0..count)
        .map(|i| {
            // 在有效範圍內均勻分布
            let ratio = (i as f64 + 0.5) / count as f64;
            let timestamp = effective_start + effective_duration * ratio;
            // 確保在有效範圍內
            timestamp.max(0.1).min(duration - 0.1)
        })
        .collect()
}

/// 選取均勻分布的時間點（帶自訂邊界）
///
/// 允許指定開始和結束的邊界比例
#[must_use]
#[allow(dead_code)]
pub fn select_uniform_timestamps_with_margin(
    duration: f64,
    count: usize,
    start_margin: f64,
    end_margin: f64,
) -> Vec<f64> {
    if count == 0 || duration <= 0.0 {
        return Vec::new();
    }

    let effective_start = duration * start_margin;
    let effective_end = duration * (1.0 - end_margin);
    let effective_duration = effective_end - effective_start;

    if effective_duration <= 0.0 {
        // 邊界太大，使用單一中間點
        return vec![duration / 2.0];
    }

    (0..count)
        .map(|i| {
            let ratio = (i as f64 + 0.5) / count as f64;
            let timestamp = effective_start + effective_duration * ratio;
            timestamp.max(0.1).min(duration - 0.1)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_uniform_timestamps_basic() {
        let timestamps = select_uniform_timestamps(100.0, 10);

        assert_eq!(timestamps.len(), 10);

        // 確保遞增
        for i in 1..timestamps.len() {
            assert!(timestamps[i] > timestamps[i - 1]);
        }

        // 確保在有效範圍內
        for t in &timestamps {
            assert!(*t >= 0.1);
            assert!(*t <= 99.9);
        }
    }

    #[test]
    fn test_select_uniform_timestamps_distribution() {
        let timestamps = select_uniform_timestamps(100.0, 5);

        // 預期值（考慮 2% 邊界）：
        // 有效範圍：2.0 ~ 98.0，長度 96.0
        // 位置：2.0 + 96.0 * (0.5/5), 2.0 + 96.0 * (1.5/5), ...
        // = 11.6, 30.8, 50.0, 69.2, 88.4

        assert_eq!(timestamps.len(), 5);
        assert!((timestamps[0] - 11.6).abs() < 0.1);
        assert!((timestamps[2] - 50.0).abs() < 0.1);
        assert!((timestamps[4] - 88.4).abs() < 0.1);
    }

    #[test]
    fn test_select_uniform_timestamps_54() {
        let timestamps = select_uniform_timestamps(3600.0, 54);

        assert_eq!(timestamps.len(), 54);

        // 確保均勻分布（間隔應該大致相等）
        let mut intervals = Vec::new();
        for i in 1..timestamps.len() {
            intervals.push(timestamps[i] - timestamps[i - 1]);
        }

        let avg_interval: f64 = intervals.iter().sum::<f64>() / intervals.len() as f64;

        // 所有間隔應該接近平均值
        for interval in &intervals {
            let diff = (interval - avg_interval).abs();
            assert!(diff < 1.0, "間隔差異過大: {diff}");
        }
    }

    #[test]
    fn test_select_uniform_timestamps_short_video() {
        let timestamps = select_uniform_timestamps(5.0, 10);

        assert_eq!(timestamps.len(), 10);

        // 確保在有效範圍內
        for t in &timestamps {
            assert!(*t >= 0.1);
            assert!(*t <= 4.9);
        }
    }

    #[test]
    fn test_select_uniform_timestamps_edge_cases() {
        assert!(select_uniform_timestamps(0.0, 10).is_empty());
        assert!(select_uniform_timestamps(100.0, 0).is_empty());
        assert!(select_uniform_timestamps(-10.0, 10).is_empty());
    }

    #[test]
    fn test_select_uniform_timestamps_with_margin() {
        let timestamps = select_uniform_timestamps_with_margin(100.0, 5, 0.1, 0.1);

        assert_eq!(timestamps.len(), 5);

        // 有效範圍：10.0 ~ 90.0
        assert!(timestamps[0] >= 10.0);
        assert!(timestamps[4] <= 90.0);
    }

    #[test]
    fn test_select_uniform_timestamps_single() {
        let timestamps = select_uniform_timestamps(100.0, 1);

        assert_eq!(timestamps.len(), 1);
        // 應該在中間附近
        assert!((timestamps[0] - 50.0).abs() < 5.0);
    }
}
