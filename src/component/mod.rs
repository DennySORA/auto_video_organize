//! 功能元件模組
//!
//! 每個子模組實現一個獨立的功能，包含主要邏輯和專用工具

pub mod auto_move_by_type;
pub mod contact_sheet_generator;
pub mod duplication_checker;
pub mod orphan_file_mover;
pub mod video_encoder;
pub mod video_renamer;

pub use auto_move_by_type::AutoMoveByType;
pub use contact_sheet_generator::ContactSheetGenerator;
pub use duplication_checker::DuplicationChecker;
pub use orphan_file_mover::OrphanFileMover;
pub use video_encoder::VideoEncoder;
pub use video_renamer::VideoRenamer;
