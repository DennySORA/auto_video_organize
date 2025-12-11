//! 影片預覽圖生成元件
//!
//! 五階段流程：
//! A. 取得影片資訊（ffprobe）
//! B. 場景變換偵測（scdet）
//! C. 選取代表時間點
//! D. 平行擷取縮圖
//! E. 合併為預覽圖

mod contact_sheet_merger;
mod main;
mod scene_detector;
mod thumbnail_extractor;
mod timestamp_selector;

pub use contact_sheet_merger::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, create_contact_sheet,
};
pub use main::{ContactSheetGenerator, GenerationResult};
pub use scene_detector::{SceneChange, SceneDetectorConfig, detect_scenes};
pub use thumbnail_extractor::{
    ThumbnailResult, ThumbnailTask, create_thumbnail_tasks, extract_thumbnail,
    extract_thumbnails_parallel,
};
pub use timestamp_selector::select_timestamps;
