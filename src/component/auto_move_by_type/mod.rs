//! 自動依類型移動檔案元件
//!
//! 掃描資料夾中的檔案，根據副檔名自動分類並移動到對應的資料夾

mod file_categorizer;
mod main;

pub use file_categorizer::{CategorizationResult, CategorizedFile, FileCategorizer};
pub use main::AutoMoveByType;
