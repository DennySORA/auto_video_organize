use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// `HashTable` 資料結構：Key 是檔案大小，Value 是該大小下所有已知檔案的 hash 集合
#[derive(Debug, Clone, Default)]
pub struct HashTable {
    entries: HashMap<u64, HashSet<String>>,
}

// 自訂序列化：將 u64 key 轉換成 string key
impl Serialize for HashTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, &HashSet<String>> = self
            .entries
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        string_map.serialize(serializer)
    }
}

// 自訂反序列化：將 string key 解析回 u64
impl<'de> Deserialize<'de> for HashTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, HashSet<String>> = HashMap::deserialize(deserializer)?;
        let entries = string_map
            .into_iter()
            .map(|(k, v)| {
                k.parse::<u64>()
                    .map(|size| (size, v))
                    .map_err(serde::de::Error::custom)
            })
            .collect::<Result<HashMap<u64, HashSet<String>>, _>>()?;
        Ok(Self { entries })
    }
}

impl HashTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("無法讀取 hash table 檔案: {}", path.display()))?;

        if content.trim().is_empty() {
            return Ok(Self::new());
        }

        serde_json::from_str(&content)
            .with_context(|| format!("無法解析 hash table 檔案: {}", path.display()))
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self).with_context(|| "無法序列化 hash table")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("無法建立目錄: {}", parent.display()))?;
        }

        fs::write(path, content)
            .with_context(|| format!("無法寫入 hash table 檔案: {}", path.display()))?;

        Ok(())
    }

    /// 檢查是否存在相同大小的檔案
    #[must_use]
    pub fn has_size(&self, size: u64) -> bool {
        self.entries.contains_key(&size)
    }

    /// 檢查特定大小下是否有特定 hash
    #[must_use]
    pub fn contains_hash(&self, size: u64, hash: &str) -> bool {
        self.entries
            .get(&size)
            .is_some_and(|hashes| hashes.contains(hash))
    }

    /// 新增一個 hash 到指定大小的列表中
    pub fn insert(&mut self, size: u64, hash: String) {
        self.entries.entry(size).or_default().insert(hash);
    }

    #[cfg(test)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_table_insert_and_contains() {
        let mut table = HashTable::new();
        table.insert(1000, "abc123".to_string());

        assert!(table.has_size(1000));
        assert!(table.contains_hash(1000, "abc123"));
        assert!(!table.contains_hash(1000, "def456"));
        assert!(!table.has_size(2000));
    }

    #[test]
    fn test_hash_table_save_and_load() {
        let mut table = HashTable::new();
        table.insert(1000, "hash1".to_string());
        table.insert(1000, "hash2".to_string());
        table.insert(2000, "hash3".to_string());

        let temp_file = NamedTempFile::new().unwrap();
        table.save_to_file(temp_file.path()).unwrap();

        let loaded = HashTable::load_from_file(temp_file.path()).unwrap();
        assert!(loaded.contains_hash(1000, "hash1"));
        assert!(loaded.contains_hash(1000, "hash2"));
        assert!(loaded.contains_hash(2000, "hash3"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let table = HashTable::load_from_file(Path::new("/nonexistent/path.json")).unwrap();
        assert!(table.is_empty());
    }
}
