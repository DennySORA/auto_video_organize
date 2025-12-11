//! E2E Integration Tests
//!
//! 測試所有功能的端對端整合

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use auto_video_organize::component::auto_move_by_type::FileCategorizer;
use auto_video_organize::component::contact_sheet_generator::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, create_contact_sheet,
    create_thumbnail_tasks, detect_scenes, extract_thumbnails_parallel, select_timestamps,
};
use auto_video_organize::component::duplication_checker::DuplicationDetector;
use auto_video_organize::component::orphan_file_mover::FileGrouper;
use auto_video_organize::config::{Config, FileCategory};
use auto_video_organize::tools::{ensure_directory_exists, get_video_info, scan_all_files};

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

/// 測試自動依類型整理檔案功能
#[test]
fn test_auto_move_by_type_e2e() {
    // 使用獨立的測試目錄
    let test_dir = Path::new("/tmp/e2e_test/auto_move_test");

    // 清理並重建測試目錄
    if test_dir.exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();

    // 建立不同類型的測試檔案
    fs::write(test_dir.join("movie.mp4"), "video content").unwrap();
    fs::write(test_dir.join("song.mp3"), "audio content").unwrap();
    fs::write(test_dir.join("photo.jpg"), "image content").unwrap();
    fs::write(test_dir.join("document.txt"), "text content").unwrap();
    fs::write(test_dir.join("archive.zip"), "archive content").unwrap();
    fs::write(test_dir.join("code.rs"), "fn main() {}").unwrap();
    fs::write(test_dir.join("unknown.xyz"), "unknown content").unwrap();

    println!("=== 測試檔案分類器 ===");

    let config = Config::new().expect("無法載入設定");
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let categorizer = FileCategorizer::new(config.file_type_table, shutdown_signal);

    // 掃描並分類
    let files = categorizer.scan_and_categorize(test_dir).unwrap();
    println!("掃描到 {} 個檔案", files.len());
    assert_eq!(files.len(), 7, "應該有 7 個檔案");

    // 驗證分類
    let video_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Video)
        .count();
    let audio_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Audio)
        .count();
    let image_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Image)
        .count();
    let document_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Document)
        .count();
    let archive_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Archive)
        .count();
    let code_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Code)
        .count();
    let other_count = files
        .iter()
        .filter(|f| f.category == FileCategory::Other)
        .count();

    println!("  Video: {video_count}");
    println!("  Audio: {audio_count}");
    println!("  Image: {image_count}");
    println!("  Document: {document_count}");
    println!("  Archive: {archive_count}");
    println!("  Code: {code_count}");
    println!("  Other: {other_count}");

    assert_eq!(video_count, 1, "應該有 1 個影片檔案");
    assert_eq!(audio_count, 1, "應該有 1 個音訊檔案");
    assert_eq!(image_count, 1, "應該有 1 個圖片檔案");
    assert_eq!(document_count, 1, "應該有 1 個文件檔案");
    assert_eq!(archive_count, 1, "應該有 1 個壓縮檔案");
    assert_eq!(code_count, 1, "應該有 1 個程式碼檔案");
    assert_eq!(other_count, 1, "應該有 1 個其他檔案");

    // 移動檔案
    println!("\n=== 移動檔案到分類資料夾 ===");
    let result = categorizer
        .move_files_to_categories(&files, test_dir)
        .unwrap();

    println!("  移動: {} 個", result.files_moved);
    println!("  失敗: {} 個", result.errors);
    assert_eq!(result.files_moved, 7, "應該移動 7 個檔案");
    assert_eq!(result.errors, 0, "不應該有錯誤");

    // 驗證檔案已移動到正確的資料夾
    assert!(
        test_dir.join("video/movie.mp4").exists(),
        "movie.mp4 應該在 video 資料夾"
    );
    assert!(
        test_dir.join("audio/song.mp3").exists(),
        "song.mp3 應該在 audio 資料夾"
    );
    assert!(
        test_dir.join("image/photo.jpg").exists(),
        "photo.jpg 應該在 image 資料夾"
    );
    assert!(
        test_dir.join("document/document.txt").exists(),
        "document.txt 應該在 document 資料夾"
    );
    assert!(
        test_dir.join("archive/archive.zip").exists(),
        "archive.zip 應該在 archive 資料夾"
    );
    assert!(
        test_dir.join("code/code.rs").exists(),
        "code.rs 應該在 code 資料夾"
    );
    assert!(
        test_dir.join("other/unknown.xyz").exists(),
        "unknown.xyz 應該在 other 資料夾"
    );

    // 驗證原始檔案已不存在
    assert!(
        !test_dir.join("movie.mp4").exists(),
        "原始 movie.mp4 應該已不存在"
    );
    assert!(
        !test_dir.join("song.mp3").exists(),
        "原始 song.mp3 應該已不存在"
    );

    println!("\n✓ 自動依類型整理檔案 E2E 測試通過");
}

/// 測試孤立檔案移動功能
#[test]
fn test_orphan_file_mover_e2e() {
    // 使用獨立的測試目錄
    let test_dir = Path::new("/tmp/e2e_test/orphan_file_test");

    // 清理並重建測試目錄
    if test_dir.exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();

    // 建立測試檔案
    // 有對應檔案的組（應保留）
    fs::write(test_dir.join("video1.mp4"), "video content 1").unwrap();
    fs::write(test_dir.join("video1.jpg"), "thumbnail 1").unwrap();

    fs::write(test_dir.join("video2.mp4"), "video content 2").unwrap();
    fs::write(test_dir.join("video2.jpg"), "thumbnail 2").unwrap();
    fs::write(test_dir.join("video2.srt"), "subtitle 2").unwrap();

    // 孤立檔案（應移動）
    fs::write(test_dir.join("orphan1.txt"), "orphan text").unwrap();
    fs::write(test_dir.join("orphan2.doc"), "orphan doc").unwrap();
    fs::write(test_dir.join("alone.mp3"), "alone audio").unwrap();

    println!("=== 測試孤立檔案移動器 ===");

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let grouper = FileGrouper::new(shutdown_signal);

    // 掃描並分組
    let groups = grouper.scan_and_group(test_dir).unwrap();
    println!("找到 {} 個群組", groups.len());
    assert_eq!(
        groups.len(),
        5,
        "應該有 5 個群組 (video1, video2, orphan1, orphan2, alone)"
    );

    // 取得孤立檔案列表
    let orphan_files = FileGrouper::get_orphan_files(&groups);
    println!("孤立檔案: {} 個", orphan_files.len());
    assert_eq!(orphan_files.len(), 3, "應該有 3 個孤立檔案");

    // 取得有對應的群組
    let paired_groups = FileGrouper::get_paired_groups(&groups);
    println!("有對應的群組: {} 個", paired_groups.len());
    assert_eq!(paired_groups.len(), 2, "應該有 2 個有對應的群組");

    // 移動孤立檔案
    println!("\n=== 移動孤立檔案 ===");
    let result = grouper.move_orphan_files(&groups, test_dir).unwrap();

    println!("  總檔案: {}", result.total_files);
    println!("  有對應（保留）: {}", result.files_with_pairs);
    println!("  孤立（已移動）: {}", result.orphan_files_moved);
    println!("  錯誤: {}", result.errors);

    assert_eq!(result.total_files, 8, "應該有 8 個檔案");
    assert_eq!(result.files_with_pairs, 5, "應該有 5 個有對應的檔案");
    assert_eq!(result.orphan_files_moved, 3, "應該移動 3 個孤立檔案");
    assert_eq!(result.errors, 0, "不應該有錯誤");

    // 驗證有對應的檔案仍在原位置
    assert!(test_dir.join("video1.mp4").exists(), "video1.mp4 應該保留");
    assert!(test_dir.join("video1.jpg").exists(), "video1.jpg 應該保留");
    assert!(test_dir.join("video2.mp4").exists(), "video2.mp4 應該保留");
    assert!(test_dir.join("video2.jpg").exists(), "video2.jpg 應該保留");
    assert!(test_dir.join("video2.srt").exists(), "video2.srt 應該保留");

    // 驗證孤立檔案已移動
    assert!(
        !test_dir.join("orphan1.txt").exists(),
        "orphan1.txt 應該已被移動"
    );
    assert!(
        !test_dir.join("orphan2.doc").exists(),
        "orphan2.doc 應該已被移動"
    );
    assert!(
        !test_dir.join("alone.mp3").exists(),
        "alone.mp3 應該已被移動"
    );

    // 驗證孤立檔案在目標目錄
    let orphan_dir = test_dir.join("orphan_files");
    assert!(orphan_dir.exists(), "孤立檔案目錄應該存在");
    assert!(
        orphan_dir.join("orphan1.txt").exists(),
        "orphan1.txt 應該在 orphan_files 目錄"
    );
    assert!(
        orphan_dir.join("orphan2.doc").exists(),
        "orphan2.doc 應該在 orphan_files 目錄"
    );
    assert!(
        orphan_dir.join("alone.mp3").exists(),
        "alone.mp3 應該在 orphan_files 目錄"
    );

    println!("\n✓ 孤立檔案移動 E2E 測試通過");
}
