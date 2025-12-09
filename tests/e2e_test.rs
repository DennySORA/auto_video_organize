//! E2E Integration Tests
//!
//! 測試所有功能的端對端整合

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use auto_video_organize::tools::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, DuplicationDetector,
    create_contact_sheet, create_thumbnail_tasks, detect_scenes, ensure_directory_exists,
    extract_thumbnails_parallel, get_video_info, scan_all_files, select_timestamps,
};

/// 測試 Duplication Checker 功能
#[test]
fn test_duplication_checker_e2e() {
    // 使用獨立的測試目錄，避免與其他測試衝突
    let test_dir = Path::new("/tmp/e2e_test/duplicates_test_isolated");
    let hash_table_path = test_dir.join("hash_table.json");

    // 清理並重建測試目錄
    if test_dir.exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();

    // 建立測試檔案 - 第一批（將被註冊）
    let file1 = test_dir.join("file1.txt");
    let file2 = test_dir.join("file2.txt");
    let file3 = test_dir.join("file3.txt");

    fs::write(&file1, "This is file 1 content").unwrap();
    fs::write(&file2, "This is file 2 content - different").unwrap();
    fs::write(&file3, "This is file 3 content - unique").unwrap();

    // 執行第一次掃描 - 註冊所有檔案
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let mut detector =
        DuplicationDetector::new(&hash_table_path, test_dir, Arc::clone(&shutdown_signal)).unwrap();

    let result1 = detector.detect_and_move_duplicates(test_dir).unwrap();

    println!("第一次掃描結果:");
    println!("  - 總檔案: {}", result1.total_files);
    println!("  - 重複檔案: {}", result1.duplicates_found);
    println!("  - 新檔案: {}", result1.new_files_registered);

    assert_eq!(result1.total_files, 3, "應該有 3 個檔案");
    assert_eq!(result1.duplicates_found, 0, "第一次掃描不應該有重複");
    assert_eq!(result1.new_files_registered, 3, "應該註冊 3 個新檔案");

    // 現在新增重複檔案
    let file1_dup = test_dir.join("file1_dup.txt");
    let file2_dup = test_dir.join("file2_dup.txt");
    fs::write(&file1_dup, "This is file 1 content").unwrap(); // 與 file1 相同
    fs::write(&file2_dup, "This is file 2 content - different").unwrap(); // 與 file2 相同

    // 執行第二次掃描 - 應該偵測到重複
    // 注意：第二次掃描會掃描到 5 個檔案（3 原始 + 2 新增）
    // 其中原始 3 個已在 hash table 中，會被識別為重複
    // 新增的 2 個因為內容相同也會被識別為重複
    let mut detector2 =
        DuplicationDetector::new(&hash_table_path, test_dir, shutdown_signal).unwrap();
    let result2 = detector2.detect_and_move_duplicates(test_dir).unwrap();

    println!("\n第二次掃描結果:");
    println!("  - 總檔案: {}", result2.total_files);
    println!("  - 重複檔案: {}", result2.duplicates_found);
    println!("  - 新檔案: {}", result2.new_files_registered);

    // 第二次掃描：
    // - file1, file2, file3 已在 hash table 中 -> 被識別為重複（3 個）
    // - file1_dup 與 file1 相同 -> 重複（1 個）
    // - file2_dup 與 file2 相同 -> 重複（1 個）
    // 總共 5 個重複
    assert_eq!(result2.duplicates_found, 5, "應該找到 5 個重複檔案");
    assert_eq!(result2.errors, 0, "不應該有錯誤");

    // 驗證重複檔案已被移動
    let dup_dir = test_dir.join("duplication_file");
    assert!(dup_dir.exists(), "重複檔案目錄應該存在");

    // 驗證 hash table 已保存
    assert!(hash_table_path.exists(), "Hash table 檔案應該存在");

    println!("\n✓ Duplication Checker E2E 測試通過");
}

/// 測試 Contact Sheet Generator 的各個階段
#[test]
fn test_contact_sheet_stages_e2e() {
    let input_dir = Path::new("/tmp/e2e_test/input");
    let output_dir = Path::new("/tmp/e2e_test/output");

    // 確保輸出目錄存在
    ensure_directory_exists(output_dir).unwrap();

    // 找到測試影片
    let video_path = input_dir.join("video_medium.mp4");
    if !video_path.exists() {
        println!("跳過測試：測試影片不存在");
        return;
    }

    println!("=== Stage A: 取得影片資訊 ===");
    let video_info = get_video_info(&video_path).unwrap();
    println!("  長度: {:.2}s", video_info.duration_seconds);
    println!("  解析度: {}x{}", video_info.width, video_info.height);
    assert!(video_info.duration_seconds > 0.0, "影片長度應該大於 0");

    println!("\n=== Stage B: 場景偵測 ===");
    let scenes = detect_scenes(&video_path, &video_info, None).unwrap();
    println!("  偵測到 {} 個場景變換點", scenes.len());
    // 我們的測試影片有 5 個場景，應該偵測到一些變換點
    // 注意：scdet 可能不會偵測到所有場景變換

    println!("\n=== Stage C: 選取時間點 ===");
    let timestamps = select_timestamps(
        video_info.duration_seconds,
        &scenes,
        DEFAULT_THUMBNAIL_COUNT,
    );
    println!("  選取了 {} 個時間點", timestamps.len());
    assert_eq!(
        timestamps.len(),
        DEFAULT_THUMBNAIL_COUNT,
        "應該選取 54 個時間點"
    );

    // 驗證時間點在有效範圍內
    for (i, &t) in timestamps.iter().enumerate() {
        assert!(
            t >= 0.0 && t <= video_info.duration_seconds,
            "時間點 {i} ({t}) 應該在有效範圍內"
        );
    }

    println!("\n=== Stage D: 擷取縮圖 ===");
    let temp_dir = output_dir.join(".tmp_test");
    ensure_directory_exists(&temp_dir).unwrap();

    let tasks = create_thumbnail_tasks(&video_path, &timestamps, &temp_dir);
    assert_eq!(tasks.len(), DEFAULT_THUMBNAIL_COUNT, "應該有 54 個任務");

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let results = extract_thumbnails_parallel(tasks, &shutdown_signal);

    let success_count = results.iter().filter(|r| r.success).count();
    println!("  成功擷取 {success_count} 張縮圖");
    assert!(
        success_count >= DEFAULT_THUMBNAIL_COUNT,
        "應該成功擷取 54 張縮圖，實際: {success_count}"
    );

    println!("\n=== Stage E: 合併預覽圖 ===");
    let mut thumbnail_paths: Vec<PathBuf> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.output_path.clone())
        .collect();
    thumbnail_paths.sort();

    let output_path = output_dir.join("test_contact_sheet.jpg");
    create_contact_sheet(
        &thumbnail_paths,
        &output_path,
        DEFAULT_GRID_COLS,
        DEFAULT_GRID_ROWS,
    )
    .unwrap();

    assert!(output_path.exists(), "預覽圖應該已建立");
    let metadata = fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0, "預覽圖檔案大小應該大於 0");

    println!("  預覽圖已建立: {}", output_path.display());
    println!("  檔案大小: {} bytes", metadata.len());

    // 清理暫存目錄
    fs::remove_dir_all(&temp_dir).unwrap();

    println!("\n✓ Contact Sheet Generator E2E 測試通過");
}

/// 測試掃描所有檔案功能
#[test]
fn test_scan_all_files_e2e() {
    let input_dir = Path::new("/tmp/e2e_test/input");

    if !input_dir.exists() {
        println!("跳過測試：測試目錄不存在");
        return;
    }

    let files = scan_all_files(input_dir).unwrap();

    println!("掃描到 {} 個檔案", files.len());

    // 驗證檔案已按大小排序（由小到大）
    for i in 1..files.len() {
        assert!(files[i].size >= files[i - 1].size, "檔案應該按大小排序");
    }

    // 應該包含我們建立的所有檔案類型
    let extensions: Vec<_> = files
        .iter()
        .filter_map(|f| f.path.extension())
        .map(|e| e.to_string_lossy().to_lowercase())
        .collect();

    assert!(extensions.contains(&"txt".to_string()), "應該包含 txt 檔案");
    assert!(extensions.contains(&"jpg".to_string()), "應該包含 jpg 檔案");
    assert!(extensions.contains(&"mp4".to_string()), "應該包含 mp4 檔案");

    println!("✓ 檔案掃描 E2E 測試通過");
}
