use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const BUFFER_SIZE: usize = 4 * 1024 * 1024; // 4MB buffer

pub fn calculate_file_hash(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("無法開啟檔案: {}", path.display()))?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("讀取檔案失敗: {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let hash = calculate_file_hash(temp_file.path()).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 produces 256-bit (64 hex chars) hash
    }

    #[test]
    fn test_same_content_same_hash() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();

        temp_file1.write_all(b"identical content").unwrap();
        temp_file2.write_all(b"identical content").unwrap();

        let hash1 = calculate_file_hash(temp_file1.path()).unwrap();
        let hash2 = calculate_file_hash(temp_file2.path()).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();

        temp_file1.write_all(b"content A").unwrap();
        temp_file2.write_all(b"content B").unwrap();

        let hash1 = calculate_file_hash(temp_file1.path()).unwrap();
        let hash2 = calculate_file_hash(temp_file2.path()).unwrap();

        assert_ne!(hash1, hash2);
    }
}
