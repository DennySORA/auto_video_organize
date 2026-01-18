//! 影片預覽圖生成元件
//!
//! 提供兩種模式：
//!
//! ## 快速模式（預設）
//! 三階段流程：
//! A. 取得影片資訊（ffprobe）
//! B. 均勻選取時間點（無需解碼）
//! C. 批次擷取縮圖並合併
//!
//! ## 精準模式
//! 五階段流程：
//! A. 取得影片資訊（ffprobe）
//! B. 場景變換偵測（scdet）
//! C. 選取代表時間點
//! D. 平行擷取縮圖
//! E. 合併為預覽圖

mod batch_extractor;
mod contact_sheet_merger;
mod main;
mod scene_detector;
mod thumbnail_extractor;
mod timestamp_selector;
mod uniform_selector;

pub use batch_extractor::{BatchExtractionResult, BatchExtractorConfig, extract_thumbnails_batch};
pub use contact_sheet_merger::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, create_contact_sheet,
};
pub use main::{ContactSheetGenerator, GenerationMode, GenerationResult};
pub use scene_detector::{SceneChange, SceneDetectorConfig, detect_scenes};
pub use thumbnail_extractor::{
    ThumbnailResult, ThumbnailTask, create_thumbnail_tasks, extract_thumbnail,
    extract_thumbnails_parallel,
};
pub use timestamp_selector::select_timestamps;
pub use uniform_selector::select_uniform_timestamps;
