use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

const MAX_FILES_SCAN: usize = 10000;
const MAX_DUPLICATES_RETURN: usize = 50;
const MIN_FILE_SIZE: u64 = 1024;
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
const MAX_DEPTH: usize = 8;

pub struct DuplicatesModule;

fn quick_hash_file(path: &Path) -> Option<u64> {
    let mut file = fs::File::open(path).ok()?;
    let mut buffer = [0u8; 4096];
    let mut hasher = u64::default();
    let mut first_bytes = [0u8; 4];

    if file.read(&mut first_bytes).ok()? > 0 {
        hasher = hasher.wrapping_add(u64::from(u32::from_ne_bytes(first_bytes)));
    }

    let mut total_read = 0;
    while total_read < 1024 * 1024 {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        total_read += n as u64;
        for (i, &byte) in buffer[..n].iter().enumerate() {
            if i % 16 == 0 {
                hasher = hasher.wrapping_mul(31).wrapping_add(u64::from(byte));
            }
        }
    }

    let metadata = fs::metadata(path).ok()?;
    hasher = hasher.wrapping_mul(31).wrapping_add(metadata.len());

    Some(hasher)
}

fn full_hash_file(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

fn scan_directory(home: &Path) -> HashMap<u64, Vec<std::path::PathBuf>> {
    let mut size_map: HashMap<u64, Vec<std::path::PathBuf>> = HashMap::new();
    let search_dirs = vec![
        home.join("Downloads"),
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Pictures"),
    ];

    let mut file_count = 0;

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if file_count >= MAX_FILES_SCAN {
            break;
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(false)
            .min_depth(1)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if file_count >= MAX_FILES_SCAN {
                break;
            }
            if !entry.file_type().is_file() {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                if size < MIN_FILE_SIZE || size > MAX_FILE_SIZE {
                    continue;
                }
                size_map
                    .entry(size)
                    .or_default()
                    .push(entry.path().to_path_buf());
                file_count += 1;
            }
        }
    }

    size_map.retain(|_, v| v.len() > 1);
    size_map
}

fn find_duplicates(home: &Path) -> Vec<(String, u64, Vec<String>)> {
    let size_map = scan_directory(home);
    let candidates: Vec<_> = size_map.into_iter().collect();

    let potential_count: usize = candidates.iter().map(|(_, v)| v.len()).sum();
    eprintln!(
        "Duplicates: {} potential groups, scanning for duplicates...",
        candidates.len()
    );

    let files_to_hash: Vec<std::path::PathBuf> = candidates
        .iter()
        .flat_map(|(_, files)| files.iter().cloned())
        .collect();

    let hashes: Vec<(String, std::path::PathBuf)> = files_to_hash
        .par_iter()
        .filter_map(|path| {
            full_hash_file(path).map(|hash| (hash, path.clone()))
        })
        .collect();

    let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();
    for (hash, path) in hashes {
        let path_str = path.to_string_lossy().to_string();
        if hash_map.contains_key(&hash) {
            hash_map.get_mut(&hash).unwrap().push(path_str);
        } else {
            hash_map.insert(hash, vec![path_str]);
        }
    }

    let mut results: Vec<(String, u64, Vec<String>)> = hash_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(hash, paths)| {
            let size = fs::metadata(&paths[0])
                .map(|m| m.len())
                .unwrap_or(0);
            let total_size = size * paths.len() as u64;
            (hash, total_size, paths)
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));

    if results.len() > MAX_DUPLICATES_RETURN {
        results.truncate(MAX_DUPLICATES_RETURN);
    }

    results
}

impl CleanModule for DuplicatesModule {
    fn id(&self) -> &'static str {
        "duplicates"
    }
    fn name(&self) -> &'static str {
        "Duplicates"
    }
    fn category(&self) -> Category {
        Category::Duplicates
    }
    fn description(&self) -> &'static str {
        "Find and remove duplicate files (by SHA-256 hash)"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let duplicates = find_duplicates(&home);
            for (hash, total_size, paths) in duplicates {
                if paths.len() >= 2 {
                    let file_size = fs::metadata(&paths[0])
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let reclaimable = file_size * (paths.len() as u64 - 1);

                    items.push(CleanItem::new(
                        &format!("dup-{}", &hash[..8]),
                        &paths[0],
                        reclaimable,
                        false,
                        &format!(
                            "{} duplicate files (save {} bytes)",
                            paths.len(),
                            total_size
                        ),
                        &format!("echo '{}' | xargs rm", paths[1..].join(" ")),
                    ));
                }
            }
        }

        Ok(items)
    }
}