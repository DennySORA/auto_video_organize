//! 資料分析紀錄與去重元件
//!
//! 使用 BLAKE3 hash 來識別重複檔案，並將重複檔案移動到指定目錄

mod duplication_detector;
mod hash_table;
mod main;

pub use duplication_detector::{DuplicationDetector, DuplicationResult};
pub use hash_table::HashTable;
pub use main::DuplicationChecker;
