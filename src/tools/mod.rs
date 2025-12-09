mod contact_sheet_merger;
mod cpu_monitor;
mod duplication_detector;
mod ffmpeg_command;
mod ffprobe_info;
mod file_hasher;
mod file_scanner;
mod hash_table;
mod path_validator;
mod scene_detector;
mod task_scheduler;
mod thumbnail_extractor;
mod timestamp_selector;
mod video_scanner;

pub use contact_sheet_merger::{
    DEFAULT_GRID_COLS, DEFAULT_GRID_ROWS, DEFAULT_THUMBNAIL_COUNT, create_contact_sheet,
};
pub use cpu_monitor::CpuMonitor;
pub use duplication_detector::{DuplicationDetector, DuplicationResult};
pub use ffmpeg_command::FfmpegCommand;
pub use ffprobe_info::{VideoInfo, get_video_info};
pub use file_hasher::calculate_file_hash;
pub use file_scanner::{FileInfo, scan_all_files};
pub use hash_table::HashTable;
pub use path_validator::{ensure_directory_exists, validate_directory_exists};
pub use scene_detector::{SceneChange, detect_scenes};
pub use task_scheduler::{EncodingTask, TaskScheduler, TaskStatus};
pub use thumbnail_extractor::{create_thumbnail_tasks, extract_thumbnails_parallel};
pub use timestamp_selector::select_timestamps;
pub use video_scanner::{VideoFileInfo, scan_video_files};
