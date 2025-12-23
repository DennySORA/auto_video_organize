//! 檔名清理模組
//!
//! 負責清理檔名中的非法字元、UUID、重複的 .convert 等

use regex::Regex;
use std::sync::LazyLock;

/// 清理後的檔名結構
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedFilename {
    /// 清理後的基本檔名（不含副檔名）
    pub base_name: String,
    /// 最終副檔名（不含前導點）
    pub extension: String,
    /// 是否包含 .convert
    pub has_convert: bool,
}

/// 檔名清理器
pub struct FilenameCleaner {
    regex_leading_number: &'static Regex,
    regex_uuid_bracket: &'static Regex,
    regex_uuid_underscore: &'static Regex,
    regex_illegal_chars: &'static Regex,
    regex_multiple_spaces: &'static Regex,
}

static REGEX_LEADING_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[\d+\]\s*").expect("Invalid regex"));

static REGEX_UUID_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\]")
        .expect("Invalid regex")
});

static REGEX_UUID_UNDERSCORE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"_[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("Invalid regex")
});

static REGEX_ILLEGAL_CHARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[<>:"/\\|?*\[\]]"#).expect("Invalid regex"));

static REGEX_MULTIPLE_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("Invalid regex"));

impl Default for FilenameCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl FilenameCleaner {
    pub fn new() -> Self {
        Self {
            regex_leading_number: &REGEX_LEADING_NUMBER,
            regex_uuid_bracket: &REGEX_UUID_BRACKET,
            regex_uuid_underscore: &REGEX_UUID_UNDERSCORE,
            regex_illegal_chars: &REGEX_ILLEGAL_CHARS,
            regex_multiple_spaces: &REGEX_MULTIPLE_SPACES,
        }
    }

    /// 清理檔名
    ///
    /// # Arguments
    /// * `filename` - 原始檔名（含副檔名）
    ///
    /// # Returns
    /// 清理後的檔名結構
    pub fn clean(&self, filename: &str) -> CleanedFilename {
        let (base_with_converts, extension) = self.split_extension(filename);
        let (base_name, has_convert) = self.extract_convert_flag(&base_with_converts);
        let cleaned_base = self.clean_base_name(&base_name);

        CleanedFilename {
            base_name: cleaned_base,
            extension,
            has_convert,
        }
    }

    /// 分離副檔名（處理 .convert.ext 的情況）
    fn split_extension(&self, filename: &str) -> (String, String) {
        let parts: Vec<&str> = filename.rsplitn(2, '.').collect();

        if parts.len() < 2 {
            return (filename.to_string(), String::new());
        }

        let extension = parts[0].to_lowercase();
        let remaining = parts[1];

        (remaining.to_string(), extension)
    }

    /// 提取 convert 標記並移除多餘的 .convert
    fn extract_convert_flag(&self, base_name: &str) -> (String, bool) {
        let mut result = base_name.to_string();
        let mut has_convert = false;

        while result.to_lowercase().ends_with(".convert") {
            has_convert = true;
            result = result[..result.len() - 8].to_string();
        }

        (result, has_convert)
    }

    /// 清理基本檔名
    fn clean_base_name(&self, base_name: &str) -> String {
        let mut result = base_name.to_string();

        result = self
            .regex_leading_number
            .replace_all(&result, "")
            .to_string();
        result = self.regex_uuid_bracket.replace_all(&result, "").to_string();
        result = self
            .regex_uuid_underscore
            .replace_all(&result, "")
            .to_string();
        result = self
            .regex_illegal_chars
            .replace_all(&result, " ")
            .to_string();
        result = self
            .regex_multiple_spaces
            .replace_all(&result, " ")
            .to_string();
        result = result.trim().to_string();

        if result.is_empty() {
            result = "video".to_string();
        }

        result
    }

    /// 產生新檔名
    ///
    /// # Arguments
    /// * `index` - 編號
    /// * `cleaned` - 清理後的檔名結構
    /// * `new_uuid` - 新的 UUID
    ///
    /// # Returns
    /// 格式化後的新檔名
    pub fn format_new_filename(
        &self,
        index: usize,
        cleaned: &CleanedFilename,
        new_uuid: &str,
    ) -> String {
        let convert_suffix = if cleaned.has_convert { ".convert" } else { "" };

        format!(
            "[{}] {}_{}{}.{}",
            index, cleaned.base_name, new_uuid, convert_suffix, cleaned.extension
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cleaner() -> FilenameCleaner {
        FilenameCleaner::new()
    }

    #[test]
    fn test_clean_simple_filename() {
        let result = cleaner().clean("my video.mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
        assert!(!result.has_convert);
    }

    #[test]
    fn test_clean_filename_with_leading_number() {
        let result = cleaner().clean("[123] my video.mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_clean_filename_with_uuid_bracket() {
        let result = cleaner().clean("my video [12345678-1234-1234-1234-123456789abc].mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_clean_filename_with_uuid_underscore() {
        let result = cleaner().clean("my video_12345678-1234-1234-1234-123456789abc.mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_clean_filename_with_convert() {
        let result = cleaner().clean("my video.convert.mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
        assert!(result.has_convert);
    }

    #[test]
    fn test_clean_filename_with_multiple_convert() {
        let result = cleaner().clean("my video.convert.convert.mp4");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "mp4");
        assert!(result.has_convert);
    }

    #[test]
    fn test_clean_filename_with_illegal_chars() {
        let result = cleaner().clean("my<>video:test.mp4");
        assert_eq!(result.base_name, "my video test");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_clean_filename_empty_base() {
        let result = cleaner().clean("[123] [12345678-1234-1234-1234-123456789abc].mp4");
        assert_eq!(result.base_name, "video");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_clean_filename_complex() {
        let result = cleaner().clean(
            "[42] my<>video:test [12345678-1234-1234-1234-123456789abc].convert.convert.mkv",
        );
        assert_eq!(result.base_name, "my video test");
        assert_eq!(result.extension, "mkv");
        assert!(result.has_convert);
    }

    #[test]
    fn test_clean_filename_chinese() {
        let result = cleaner().clean("[1] 中文影片名稱.mp4");
        assert_eq!(result.base_name, "中文影片名稱");
        assert_eq!(result.extension, "mp4");
    }

    #[test]
    fn test_format_new_filename() {
        let cleaned = CleanedFilename {
            base_name: "my video".to_string(),
            extension: "mp4".to_string(),
            has_convert: false,
        };
        let result =
            cleaner().format_new_filename(1, &cleaned, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert_eq!(
            result,
            "[1] my video_aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.mp4"
        );
    }

    #[test]
    fn test_format_new_filename_with_convert() {
        let cleaned = CleanedFilename {
            base_name: "my video".to_string(),
            extension: "mp4".to_string(),
            has_convert: true,
        };
        let result =
            cleaner().format_new_filename(1, &cleaned, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert_eq!(
            result,
            "[1] my video_aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.convert.mp4"
        );
    }

    #[test]
    fn test_clean_filename_no_extension() {
        let result = cleaner().clean("my video");
        assert_eq!(result.base_name, "my video");
        assert_eq!(result.extension, "");
    }

    #[test]
    fn test_clean_filename_multiple_spaces() {
        let result = cleaner().clean("my    video   test.mp4");
        assert_eq!(result.base_name, "my video test");
    }
}
