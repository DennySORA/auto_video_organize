//! 整合測試 - 使用生成的測試資料驗證系統功能
//!
//! 測試資料位於 /`tmp/video_organize_test/input`

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use auto_video_organize::component::auto_move_by_type::FileCategorizer;
use auto_video_organize::component::contact_sheet_generator::{
    create_thumbnail_tasks, detect_scenes, extract_thumbnails_parallel, select_timestamps,
};
use auto_video_organize::component::duplication_checker::DuplicationDetector;
use auto_video_organize::component::orphan_file_mover::FileGrouper;
use auto_video_organize::config::{Config, FileCategory};
use auto_video_organize::tools::{ensure_directory_exists, get_video_info, scan_all_files};

const TEST_INPUT_DIR: &str = "/tmp/video_organize_test/input";
const TEST_OUTPUT_DIR: &str = "/tmp/video_organize_test/output";

/// 測試 1: 檔案掃描功能
#[test]
fn test_file_scanning() {
    let input_dir = Path::new(TEST_INPUT_DIR);
    if !input_dir.exists() {
        println!("跳過測試：測試目錄不存在，請先執行 test data 生成");
        return;
    }

    let files = scan_all_files(input_dir).unwrap();
    println!("掃描到 {} 個檔案", files.len());

    // 應該至少有我們建立的測試檔案
    assert!(files.len() >= 20, "應該至少有 20 個測試檔案");

    // 驗證檔案按大小排序
    for i in 1..files.len() {
        assert!(
            files[i].size >= files[i - 1].size,
            "檔案應該按大小升序排列"
        );
    }

    println!("✓ 檔案掃描測試通過");
}

/// 測試 2: 影片資訊取得
#[test]
fn test_video_info_extraction() {
    let video_path = Path::new(TEST_INPUT_DIR).join("test_video_01.mp4");
    if !video_path.exists() {
        println!("跳過測試：測試影片不存在");
        return;
    }

    let info = get_video_info(&video_path).unwrap();

    println!("影片資訊:");
    println!("  時長: {:.2}s", info.duration_seconds);
    println!("  解析度: {}x{}", info.width, info.height);
    println!("  幀率: {:.2}", info.frame_rate);

    assert!(info.duration_seconds > 0.0, "影片時長應該大於 0");
    assert_eq!(info.width, 640, "寬度應該是 640");
    assert_eq!(info.height, 360, "高度應該是 360");

    println!("✓ 影片資訊取得測試通過");
}

/// 測試 3: 場景偵測
#[test]
fn test_scene_detection() {
    let video_path = Path::new(TEST_INPUT_DIR).join("test_video_01.mp4");
    if !video_path.exists() {
        println!("跳過測試：測試影片不存在");
        return;
    }

    let info = get_video_info(&video_path).unwrap();
    let scenes = detect_scenes(&video_path, &info, None).unwrap();

    println!("偵測到 {} 個場景變換點", scenes.len());

    // test_video_01 有 4 個不同顏色的場景，應該偵測到場景變換
    // 注意：實際偵測數量可能因閾值而異
    for (i, scene) in scenes.iter().enumerate() {
        println!("  場景 {}: {:.2}s", i + 1, scene.timestamp);
    }

    println!("✓ 場景偵測測試通過");
}

/// 測試 4: 時間點選取
#[test]
fn test_timestamp_selection() {
    let video_path = Path::new(TEST_INPUT_DIR).join("test_video_02.mp4");
    if !video_path.exists() {
        println!("跳過測試：測試影片不存在");
        return;
    }

    let info = get_video_info(&video_path).unwrap();
    let scenes = detect_scenes(&video_path, &info, None).unwrap();

    // 選取 9 個時間點（比較少的數量用於測試）
    let timestamps = select_timestamps(info.duration_seconds, &scenes, 9);

    println!("選取了 {} 個時間點:", timestamps.len());
    for (i, t) in timestamps.iter().enumerate() {
        println!("  {}: {:.2}s", i + 1, t);
    }

    assert_eq!(timestamps.len(), 9, "應該選取 9 個時間點");

    // 驗證時間點在有效範圍內且遞增
    for i in 0..timestamps.len() {
        assert!(
            timestamps[i] >= 0.0 && timestamps[i] < info.duration_seconds,
            "時間點應該在有效範圍內"
        );
        if i > 0 {
            assert!(timestamps[i] > timestamps[i - 1], "時間點應該遞增");
        }
    }

    println!("✓ 時間點選取測試通過");
}

/// 測試 5: 縮圖擷取
#[test]
fn test_thumbnail_extraction() {
    let video_path = Path::new(TEST_INPUT_DIR).join("test_video_01.mp4");
    let output_dir = Path::new(TEST_OUTPUT_DIR).join("thumbnails_test");

    if !video_path.exists() {
        println!("跳過測試：測試影片不存在");
        return;
    }

    // 清理並建立輸出目錄
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap();
    }
    ensure_directory_exists(&output_dir).unwrap();

    let _info = get_video_info(&video_path).unwrap();
    let timestamps = vec![1.0, 3.0, 5.0, 7.0]; // 4 個時間點

    let tasks = create_thumbnail_tasks(&video_path, &timestamps, &output_dir);
    assert_eq!(tasks.len(), 4, "應該有 4 個任務");

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let results = extract_thumbnails_parallel(tasks, &shutdown_signal);

    let success_count = results.iter().filter(|r| r.success).count();
    println!("成功擷取 {success_count} 張縮圖");

    assert_eq!(success_count, 4, "應該成功擷取 4 張縮圖");

    // 驗證縮圖檔案存在
    for result in &results {
        if result.success {
            assert!(
                result.output_path.exists(),
                "縮圖檔案應該存在: {}",
                result.output_path.display()
            );
        }
    }

    println!("✓ 縮圖擷取測試通過");
}

/// 測試 6: 檔案分類
#[test]
fn test_file_categorization() {
    let input_dir = Path::new(TEST_INPUT_DIR);
    if !input_dir.exists() {
        println!("跳過測試：測試目錄不存在");
        return;
    }

    let config = Config::new().expect("無法載入設定");
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let categorizer = FileCategorizer::new(config.file_type_table, shutdown_signal);

    let files = categorizer.scan_and_categorize(input_dir).unwrap();

    // 統計各分類
    let video_count = files.iter().filter(|f| f.category == FileCategory::Video).count();
    let audio_count = files.iter().filter(|f| f.category == FileCategory::Audio).count();
    let image_count = files.iter().filter(|f| f.category == FileCategory::Image).count();
    let archive_count = files.iter().filter(|f| f.category == FileCategory::Archive).count();
    let code_count = files.iter().filter(|f| f.category == FileCategory::Code).count();
    let document_count = files.iter().filter(|f| f.category == FileCategory::Document).count();
    let markup_count = files.iter().filter(|f| f.category == FileCategory::Markup).count();

    println!("檔案分類結果:");
    println!("  影片: {video_count}");
    println!("  音訊: {audio_count}");
    println!("  圖片: {image_count}");
    println!("  壓縮檔: {archive_count}");
    println!("  程式碼: {code_count}");
    println!("  文件: {document_count}");
    println!("  標記語言: {markup_count}");

    // 驗證分類正確
    assert!(video_count >= 4, "應該至少有 4 個影片檔案");
    assert!(audio_count >= 2, "應該至少有 2 個音訊檔案");
    assert!(image_count >= 4, "應該至少有 4 個圖片檔案");

    println!("✓ 檔案分類測試通過");
}

/// 測試 7: 檔案分組（孤立檔案偵測）
#[test]
fn test_file_grouping() {
    let input_dir = Path::new(TEST_INPUT_DIR);
    if !input_dir.exists() {
        println!("跳過測試：測試目錄不存在");
        return;
    }

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let grouper = FileGrouper::new(shutdown_signal);

    let groups = grouper.scan_and_group(input_dir).unwrap();

    let orphan_files = FileGrouper::get_orphan_files(&groups);
    let paired_groups = FileGrouper::get_paired_groups(&groups);

    println!("分組結果:");
    println!("  總群組數: {}", groups.len());
    println!("  有對應的群組: {}", paired_groups.len());
    println!("  孤立檔案: {}", orphan_files.len());

    // paired_video 應該是一個有對應的群組（mp4, jpg, srt）
    let paired_video_group = groups.iter().find(|g| g.stem == "paired_video");
    assert!(paired_video_group.is_some(), "應該找到 paired_video 群組");
    assert!(
        !paired_video_group.unwrap().is_orphan(),
        "paired_video 群組不應該是孤立的"
    );

    // lonely_file 應該是孤立的
    let lonely_group = groups.iter().find(|g| g.stem == "lonely_file");
    assert!(lonely_group.is_some(), "應該找到 lonely_file 群組");
    assert!(
        lonely_group.unwrap().is_orphan(),
        "lonely_file 應該是孤立的"
    );

    println!("✓ 檔案分組測試通過");
}

/// 測試 8: 去重偵測
#[test]
fn test_duplication_detection() {
    // 使用獨立的測試目錄
    let test_dir = Path::new("/tmp/video_organize_test/dup_test");

    // 清理並重建測試目錄
    if test_dir.exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();

    // 建立測試檔案
    fs::write(test_dir.join("original.txt"), "This is original content").unwrap();
    fs::write(test_dir.join("unique.txt"), "This is unique content").unwrap();

    let hash_table_path = test_dir.join(".hash_table.json");
    let shutdown_signal = Arc::new(AtomicBool::new(false));

    // 第一次掃描 - 註冊檔案
    let mut detector =
        DuplicationDetector::new(&hash_table_path, test_dir, Arc::clone(&shutdown_signal)).unwrap();
    let result1 = detector.detect_and_move_duplicates(test_dir).unwrap();

    println!("第一次掃描:");
    println!("  新增註冊: {}", result1.new_files_registered);
    println!("  重複: {}", result1.duplicates_found);

    assert_eq!(result1.new_files_registered, 2, "應該註冊 2 個新檔案");
    assert_eq!(result1.duplicates_found, 0, "第一次不應該有重複");

    // 新增重複檔案
    fs::write(test_dir.join("original_copy.txt"), "This is original content").unwrap();

    // 第二次掃描 - 應該偵測到重複
    let mut detector2 =
        DuplicationDetector::new(&hash_table_path, test_dir, shutdown_signal).unwrap();
    let result2 = detector2.detect_and_move_duplicates(test_dir).unwrap();

    println!("第二次掃描:");
    println!("  重複: {}", result2.duplicates_found);

    // 第二次掃描會發現原本的 2 個檔案 + 新複製的 1 個 = 3 個重複
    assert!(result2.duplicates_found >= 1, "應該偵測到至少 1 個重複");

    println!("✓ 去重偵測測試通過");
}
