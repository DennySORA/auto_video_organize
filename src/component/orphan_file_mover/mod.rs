//! 孤立檔案移動元件
//!
//! 找出沒有對應檔案的孤立檔案（例如只有預覽圖沒有對應影片的情況），
//! 並將其移動到指定目錄

mod file_grouper;
mod main;

pub use file_grouper::{FileGroup, FileGrouper, OrphanMoveResult};
pub use main::OrphanFileMover;
