use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

/// 支援的語言
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Language {
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "zh-TW")]
    #[default]
    ZhTw,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "ja-JP")]
    JaJp,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnUs => write!(f, "en-US"),
            Self::ZhTw => write!(f, "zh-TW"),
            Self::ZhCn => write!(f, "zh-CN"),
            Self::JaJp => write!(f, "ja-JP"),
        }
    }
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::ZhTw => "zh-TW",
            Self::ZhCn => "zh-CN",
            Self::JaJp => "ja-JP",
        }
    }
}

/// 轉檔後處理動作
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PostEncodeAction {
    /// 不移動任何檔案（預設）
    #[default]
    #[serde(rename = "none")]
    None,
    /// 移動舊影片（原始檔案）到 finish 資料夾
    #[serde(rename = "move_old_to_finish")]
    MoveOldToFinish,
    /// 移動新影片（轉檔後檔案）到 finish 資料夾
    #[serde(rename = "move_new_to_finish")]
    MoveNewToFinish,
}

impl fmt::Display for PostEncodeAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "不移動"),
            Self::MoveOldToFinish => write!(f, "移動舊影片到 finish"),
            Self::MoveNewToFinish => write!(f, "移動新影片到 finish"),
        }
    }
}

/// 縮圖輸出模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ContactSheetOutputMode {
    /// 輸出到子目錄（預設，向後兼容）
    #[default]
    #[serde(rename = "sub_directory")]
    SubDirectory,
    /// 輸出到影片同目錄
    #[serde(rename = "same_directory")]
    SameDirectory,
}

impl fmt::Display for ContactSheetOutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubDirectory => write!(f, "子目錄"),
            Self::SameDirectory => write!(f, "同目錄"),
        }
    }
}

/// 縮圖產生設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactSheetSettings {
    /// 輸出模式
    pub output_mode: ContactSheetOutputMode,
}

/// 影片轉檔設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoEncoderSettings {
    /// 轉檔後處理動作
    pub post_encode_action: PostEncodeAction,
    /// 初始最大轉檔數（None = CPU 1/4）
    #[serde(default = "VideoEncoderSettings::default_initial_limit")]
    pub initial_max_parallel: Option<usize>,
    /// 最大同時轉檔數（None = 無上限）
    #[serde(default = "VideoEncoderSettings::default_max_parallel")]
    pub max_parallel: Option<usize>,
}

impl VideoEncoderSettings {
    const fn default_initial_limit() -> Option<usize> {
        None
    }
    const fn default_max_parallel() -> Option<usize> {
        None
    }
}

/// 最近使用路徑的最大數量
pub const MAX_RECENT_PATHS: usize = 10;

/// 使用者設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    /// 語言設定
    pub language: Language,
    /// 影片轉檔設定
    #[serde(default)]
    pub video_encoder: VideoEncoderSettings,
    /// 縮圖產生設定
    #[serde(default)]
    pub contact_sheet: ContactSheetSettings,
    /// 最近使用的路徑（最多 10 個）
    #[serde(default)]
    pub recent_paths: Vec<String>,
}

/// 檔案類型分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Video,
    Audio,
    Image,
    Archive,
    Document,
    Spreadsheet,
    Presentation,
    Ebook,
    Code,
    Markup,
    Database,
    Executable,
    Font,
    Cad3D,
    System,
    Other,
}

impl FileCategory {
    /// 取得分類的資料夾名稱
    #[must_use]
    pub const fn folder_name(&self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Archive => "archive",
            Self::Document => "document",
            Self::Spreadsheet => "spreadsheet",
            Self::Presentation => "presentation",
            Self::Ebook => "ebook",
            Self::Code => "code",
            Self::Markup => "markup",
            Self::Database => "database",
            Self::Executable => "executable",
            Self::Font => "font",
            Self::Cad3D => "cad_3d",
            Self::System => "system",
            Self::Other => "other",
        }
    }

    /// 取得分類的顯示名稱
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Video => "影片",
            Self::Audio => "音訊",
            Self::Image => "圖片",
            Self::Archive => "壓縮檔",
            Self::Document => "文件",
            Self::Spreadsheet => "試算表",
            Self::Presentation => "簡報",
            Self::Ebook => "電子書",
            Self::Code => "程式碼",
            Self::Markup => "標記語言",
            Self::Database => "資料庫",
            Self::Executable => "執行檔",
            Self::Font => "字型",
            Self::Cad3D => "CAD/3D",
            Self::System => "系統檔",
            Self::Other => "其他",
        }
    }

    /// 取得所有分類（不含 Other）
    #[must_use]
    pub const fn all_categories() -> &'static [Self] {
        &[
            Self::Video,
            Self::Audio,
            Self::Image,
            Self::Archive,
            Self::Document,
            Self::Spreadsheet,
            Self::Presentation,
            Self::Ebook,
            Self::Code,
            Self::Markup,
            Self::Database,
            Self::Executable,
            Self::Font,
            Self::Cad3D,
            Self::System,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeTable {
    #[serde(rename = "VIDEO_FILE")]
    pub video_file: Vec<String>,
    #[serde(rename = "AUDIO_FILE")]
    pub audio_file: Vec<String>,
    #[serde(rename = "IMAGE_FILE")]
    pub image_file: Vec<String>,
    #[serde(rename = "ARCHIVE_FILE")]
    pub archive_file: Vec<String>,
    #[serde(rename = "DOCUMENT_FILE")]
    pub document_file: Vec<String>,
    #[serde(rename = "SPREADSHEET_FILE")]
    pub spreadsheet_file: Vec<String>,
    #[serde(rename = "PRESENTATION_FILE")]
    pub presentation_file: Vec<String>,
    #[serde(rename = "EBOOK_FILE")]
    pub ebook_file: Vec<String>,
    #[serde(rename = "CODE_FILE")]
    pub code_file: Vec<String>,
    #[serde(rename = "MARKUP_LANGUAGE_FILE")]
    pub markup_language_file: Vec<String>,
    #[serde(rename = "DATABASE_FILE")]
    pub database_file: Vec<String>,
    #[serde(rename = "EXECUTABLE_FILE")]
    pub executable_file: Vec<String>,
    #[serde(rename = "FONT_FILE")]
    pub font_file: Vec<String>,
    #[serde(rename = "CAD_3D_FILE")]
    pub cad_3d_file: Vec<String>,
    #[serde(rename = "SYSTEM_FILE")]
    pub system_file: Vec<String>,
}

impl FileTypeTable {
    /// 取得指定分類的副檔名集合
    #[must_use]
    pub fn extensions_for_category(&self, category: FileCategory) -> HashSet<String> {
        let extensions = match category {
            FileCategory::Video => &self.video_file,
            FileCategory::Audio => &self.audio_file,
            FileCategory::Image => &self.image_file,
            FileCategory::Archive => &self.archive_file,
            FileCategory::Document => &self.document_file,
            FileCategory::Spreadsheet => &self.spreadsheet_file,
            FileCategory::Presentation => &self.presentation_file,
            FileCategory::Ebook => &self.ebook_file,
            FileCategory::Code => &self.code_file,
            FileCategory::Markup => &self.markup_language_file,
            FileCategory::Database => &self.database_file,
            FileCategory::Executable => &self.executable_file,
            FileCategory::Font => &self.font_file,
            FileCategory::Cad3D => &self.cad_3d_file,
            FileCategory::System => &self.system_file,
            FileCategory::Other => return HashSet::new(),
        };
        extensions.iter().map(|ext| ext.to_lowercase()).collect()
    }

    /// 判斷檔案屬於哪個分類
    #[must_use]
    pub fn categorize_file(&self, path: &Path) -> FileCategory {
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => format!(".{}", e.to_lowercase()),
            None => return FileCategory::Other,
        };

        // 按優先順序檢查各分類
        for &category in FileCategory::all_categories() {
            if self.extensions_for_category(category).contains(&ext) {
                return category;
            }
        }

        FileCategory::Other
    }

    #[must_use]
    pub fn video_extensions_set(&self) -> HashSet<String> {
        self.extensions_for_category(FileCategory::Video)
    }

    #[must_use]
    pub fn is_video_file(&self, path: &Path) -> bool {
        self.categorize_file(path) == FileCategory::Video
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub file_type_table: FileTypeTable,
    pub settings: UserSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file_type_table() -> FileTypeTable {
        FileTypeTable {
            video_file: vec![".mp4".to_string(), ".mkv".to_string()],
            audio_file: vec![".mp3".to_string(), ".flac".to_string()],
            image_file: vec![".jpg".to_string(), ".png".to_string()],
            archive_file: vec![".zip".to_string(), ".rar".to_string()],
            document_file: vec![".doc".to_string(), ".txt".to_string()],
            spreadsheet_file: vec![".xls".to_string(), ".csv".to_string()],
            presentation_file: vec![".ppt".to_string()],
            ebook_file: vec![".epub".to_string()],
            code_file: vec![".rs".to_string(), ".py".to_string()],
            markup_language_file: vec![".html".to_string(), ".json".to_string()],
            database_file: vec![".db".to_string(), ".sqlite".to_string()],
            executable_file: vec![".exe".to_string()],
            font_file: vec![".ttf".to_string()],
            cad_3d_file: vec![".obj".to_string()],
            system_file: vec![".dll".to_string()],
        }
    }

    #[test]
    fn test_categorize_video_file() {
        let table = create_test_file_type_table();
        assert_eq!(
            table.categorize_file(Path::new("movie.mp4")),
            FileCategory::Video
        );
        assert_eq!(
            table.categorize_file(Path::new("movie.MKV")),
            FileCategory::Video
        );
    }

    #[test]
    fn test_categorize_image_file() {
        let table = create_test_file_type_table();
        assert_eq!(
            table.categorize_file(Path::new("photo.jpg")),
            FileCategory::Image
        );
        assert_eq!(
            table.categorize_file(Path::new("photo.PNG")),
            FileCategory::Image
        );
    }

    #[test]
    fn test_categorize_other_file() {
        let table = create_test_file_type_table();
        assert_eq!(
            table.categorize_file(Path::new("unknown.xyz")),
            FileCategory::Other
        );
        assert_eq!(
            table.categorize_file(Path::new("noextension")),
            FileCategory::Other
        );
    }

    #[test]
    fn test_folder_name() {
        assert_eq!(FileCategory::Video.folder_name(), "video");
        assert_eq!(FileCategory::Image.folder_name(), "image");
        assert_eq!(FileCategory::Other.folder_name(), "other");
    }
}
