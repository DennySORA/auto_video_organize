use anyhow::Result;
use std::collections::HashMap;

struct Config {
    file_type: HashMap<String, Vec<String>>,
}

impl Config {
    fn new() -> Result<Self> {
        Ok(Self {
            file_type: Self::load_file_types()?,
        })
    }

    fn load_file_types() -> Result<HashMap<String, Vec<String>>> {
        Ok(serde_json::from_str(include_str!("../data/file_type_table.json"))?)
    }
}
