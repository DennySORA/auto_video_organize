use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

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
    #[must_use] 
    pub fn video_extensions_set(&self) -> HashSet<String> {
        self.video_file
            .iter()
            .map(|ext| ext.to_lowercase())
            .collect()
    }

    #[must_use] 
    pub fn is_video_file(&self, path: &Path) -> bool {
        let video_extensions = self.video_extensions_set();
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| video_extensions.contains(&format!(".{}", ext.to_lowercase())))
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub file_type_table: FileTypeTable,
}
