use crate::tools::SceneChange;

/// 從場景變換點中選取指定數量的代表時間點
///
/// 策略：
/// 1. 將場景變換點轉換為片段（segments）
/// 2. 如果片段數量 >= count：均勻選取 count 個片段
/// 3. 如果片段數量 < count：對最長的片段進行二分切割直到達到 count
/// 4. 每個片段選取 35% 處作為代表時間點（避開轉場邊界）
#[must_use]
pub fn select_timestamps(duration: f64, scene_changes: &[SceneChange], count: usize) -> Vec<f64> {
    if count == 0 || duration <= 0.0 {
        return Vec::new();
    }

    // 建立片段列表
    let mut segments = build_segments(duration, scene_changes);

    // 調整片段數量以匹配 count
    if segments.len() > count {
        // 片段太多，均勻抽取
        segments = select_evenly(&segments, count);
    } else if segments.len() < count {
        // 片段不足，切割最長片段補足
        segments = split_longest_segments(segments, count);
    }

    // 從每個片段中選取代表時間點
    segments
        .iter()
        .take(count)
        .map(|seg| calculate_representative_time(seg.0, seg.1, duration))
        .collect()
}

/// 從場景變換點建立片段列表
fn build_segments(duration: f64, scene_changes: &[SceneChange]) -> Vec<(f64, f64)> {
    let mut points: Vec<f64> = vec![0.0];
    points.extend(scene_changes.iter().map(|sc| sc.timestamp));
    points.push(duration);

    // 去重並排序
    points.sort_by(|a, b| a.partial_cmp(b).unwrap());
    points.dedup_by(|a, b| (*a - *b).abs() < 0.1);

    // 建立片段，過濾掉太短的片段（< 0.5 秒）
    points
        .windows(2)
        .filter_map(|w| {
            let (start, end) = (w[0], w[1]);
            if end - start >= 0.5 {
                Some((start, end))
            } else {
                None
            }
        })
        .collect()
}

/// 均勻選取片段
fn select_evenly(segments: &[(f64, f64)], count: usize) -> Vec<(f64, f64)> {
    if segments.is_empty() || count == 0 {
        return Vec::new();
    }

    let step = (segments.len() - 1) as f64 / (count - 1).max(1) as f64;

    (0..count)
        .map(|i| {
            let index = ((i as f64) * step).round() as usize;
            segments[index.min(segments.len() - 1)]
        })
        .collect()
}

/// 切割最長片段直到達到目標數量
fn split_longest_segments(mut segments: Vec<(f64, f64)>, target_count: usize) -> Vec<(f64, f64)> {
    while segments.len() < target_count {
        // 找到最長的片段
        let longest_idx = segments
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let len_a = a.1 - a.0;
                let len_b = b.1 - b.0;
                len_a.partial_cmp(&len_b).unwrap()
            })
            .map_or(0, |(i, _)| i);

        let (start, end) = segments[longest_idx];
        let mid = f64::midpoint(start, end);

        // 替換為兩個子片段
        segments[longest_idx] = (start, mid);
        segments.insert(longest_idx + 1, (mid, end));
    }

    // 確保按時間順序排列
    segments.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    segments
}

/// 計算片段的代表時間點
/// 選取片段內 35% 處，避開轉場邊界
fn calculate_representative_time(start: f64, end: f64, duration: f64) -> f64 {
    let segment_length = end - start;

    // 在片段內 35% 處，但確保至少離邊界 0.5 秒
    let offset = (segment_length * 0.35).max(0.5).min(segment_length - 0.5);
    let time = start + offset.max(0.0);

    // 確保在影片範圍內
    time.max(0.0).min(duration - 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene_change(timestamp: f64) -> SceneChange {
        SceneChange {
            timestamp,
            score: 1.0,
        }
    }

    #[test]
    fn test_select_timestamps_exact_count() {
        let duration = 100.0;
        let scenes: Vec<SceneChange> = (1..=5)
            .map(|i| make_scene_change(f64::from(i) * 10.0))
            .collect();

        let timestamps = select_timestamps(duration, &scenes, 6);
        assert_eq!(timestamps.len(), 6);

        // 確保時間點在有效範圍內
        for t in &timestamps {
            assert!(*t >= 0.0 && *t < duration);
        }
    }

    #[test]
    fn test_select_timestamps_more_scenes() {
        let duration = 100.0;
        let scenes: Vec<SceneChange> = (1..=20)
            .map(|i| make_scene_change(f64::from(i) * 4.0))
            .collect();

        let timestamps = select_timestamps(duration, &scenes, 5);
        assert_eq!(timestamps.len(), 5);

        // 確保均勻分布
        for i in 1..timestamps.len() {
            assert!(timestamps[i] > timestamps[i - 1]);
        }
    }

    #[test]
    fn test_select_timestamps_fewer_scenes() {
        let duration = 100.0;
        let scenes = vec![make_scene_change(50.0)];

        let timestamps = select_timestamps(duration, &scenes, 4);
        assert_eq!(timestamps.len(), 4);

        // 確保時間點是遞增的
        for i in 1..timestamps.len() {
            assert!(timestamps[i] > timestamps[i - 1]);
        }
    }

    #[test]
    fn test_select_timestamps_no_scenes() {
        let duration = 100.0;
        let scenes: Vec<SceneChange> = vec![];

        let timestamps = select_timestamps(duration, &scenes, 54);
        assert_eq!(timestamps.len(), 54);

        // 確保時間點是遞增的
        for i in 1..timestamps.len() {
            assert!(timestamps[i] > timestamps[i - 1]);
        }
    }

    #[test]
    fn test_select_timestamps_edge_cases() {
        assert!(select_timestamps(0.0, &[], 10).is_empty());
        assert!(select_timestamps(100.0, &[], 0).is_empty());
    }

    #[test]
    fn test_build_segments() {
        let scenes = vec![make_scene_change(10.0), make_scene_change(20.0)];
        let segments = build_segments(30.0, &scenes);

        assert_eq!(segments.len(), 3);
        assert!((segments[0].0 - 0.0).abs() < 0.01);
        assert!((segments[0].1 - 10.0).abs() < 0.01);
        assert!((segments[1].0 - 10.0).abs() < 0.01);
        assert!((segments[1].1 - 20.0).abs() < 0.01);
        assert!((segments[2].0 - 20.0).abs() < 0.01);
        assert!((segments[2].1 - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_split_longest_segments() {
        let segments = vec![(0.0, 10.0), (10.0, 20.0)];
        let result = split_longest_segments(segments, 4);

        assert_eq!(result.len(), 4);
        // 確保排序正確
        for i in 1..result.len() {
            assert!(result[i].0 >= result[i - 1].1 - 0.01);
        }
    }
}
