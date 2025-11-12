use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

fn update_entry(
    map: &mut HashMap<OsString, (PathBuf, u64)>,
    name: OsString,
    path: PathBuf,
    size: u64,
) {
    map.entry(name)
        .and_modify(|e| {
            if size > e.1 {
                *e = (path.clone(), size);
            }
        })
        .or_insert((path, size));
}

fn process_entry(
    mut map: HashMap<OsString, (PathBuf, u64)>,
    entry: DirEntry,
) -> HashMap<OsString, (PathBuf, u64)> {
    if let Ok(meta) = std::fs::symlink_metadata(entry.path()) {
        update_entry(
            &mut map,
            entry.file_name().to_owned(),
            entry.into_path(),
            meta.len(),
        );
    }
    map
}

fn merge_maps(
    mut acc: HashMap<OsString, (PathBuf, u64)>,
    other: HashMap<OsString, (PathBuf, u64)>,
) -> HashMap<OsString, (PathBuf, u64)> {
    for (name, (path, size)) in other {
        update_entry(&mut acc, name, path, size);
    }
    acc
}

pub fn get_file_map(path: &str) -> HashMap<OsString, (PathBuf, u64)> {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .par_bridge()
        .fold(HashMap::new, process_entry)
        .reduce(HashMap::new, merge_maps)
}
