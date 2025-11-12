use anyhow::Result;
use std::io::{BufReader, Read, Seek, SeekFrom};

pub trait HashExt {
    fn calculate_hash(&mut self) -> Result<String>;
}

impl<T: Read + Seek> HashExt for T {
    fn calculate_hash(&mut self) -> Result<String> {
        let current_pos = match self.stream_position() {
            Ok(p) => p,
            Err(_) => self.seek(SeekFrom::Current(0))?,
        };

        self.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::with_capacity(4 * 1024 * 1024, self);
        let mut hasher = blake3::Hasher::new();
        let mut buf = vec![0u8; 4 * 1024 * 1024];

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        reader.into_inner().seek(SeekFrom::Start(current_pos))?;
        Ok(hasher.finalize().to_hex().to_string())
    }
}
