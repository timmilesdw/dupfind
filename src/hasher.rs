use anyhow::Result;
use blake3::Hasher;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::utils::INTERRUPTED;

pub fn quick_hash_file(path: &Path, sample_size: usize, buffer_size: usize) -> Result<String> {
    let mut file = BufReader::with_capacity(buffer_size * 1024, File::open(path)?);
    let mut buffer = vec![0u8; sample_size];
    let bytes_read = file.read(&mut buffer)?;

    let mut hasher = Hasher::new();
    hasher.update(&buffer[..bytes_read]);
    Ok(hasher.finalize().to_string())
}

pub fn full_hash_file(path: &Path, buffer_size: usize) -> Result<String> {
    let mut file = BufReader::with_capacity(buffer_size * 1024 * 1024, File::open(path)?);
    let mut hasher = Hasher::new();

    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_string())
}

pub fn compute_hashes(
    groups: HashMap<u64, Vec<std::path::PathBuf>>,
    quick_hash_size: usize,
    quick_buffer_size: usize,
    full_buffer_size: usize,
    progress: &ProgressBar,
) -> Result<HashMap<String, Vec<std::path::PathBuf>>> {
    let processed = Arc::new(AtomicU64::new(0));
    let total: u64 = groups.values().map(|files| files.len() as u64).sum();

    let hash_results: Vec<_> = groups
        .into_par_iter()
        .filter(|(_, files)| files.len() >= 2)
        .flat_map(|(_size, files)| {
            if INTERRUPTED.load(Ordering::Relaxed) {
                return Vec::new();
            }
            let quick_hashes: Vec<_> = files
                .par_iter()
                .filter_map(|path| {
                    quick_hash_file(path, quick_hash_size, quick_buffer_size)
                        .map(|hash| (hash, path.clone()))
                        .ok()
                })
                .collect();
            let mut quick_groups: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();
            for (hash, path) in quick_hashes {
                quick_groups.entry(hash).or_default().push(path);
            }
            quick_groups
                .into_par_iter()
                .filter(|(_, paths)| paths.len() >= 2)
                .flat_map(|(_, paths)| {
                    paths
                        .par_iter()
                        .filter_map(|path| {
                            let result = full_hash_file(path, full_buffer_size)
                                .map(|hash| (hash, path.clone()));

                            let current = processed.fetch_add(1, Ordering::Relaxed);
                            if current.is_multiple_of(100) {
                                progress.set_position(current.min(total));
                            }

                            result.ok()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    progress.set_position(total);

    let mut hashes: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();
    for (hash, path) in hash_results {
        hashes.entry(hash).or_default().push(path);
    }

    hashes.retain(|_, files| files.len() > 1);
    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_quick_hash_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let hash = quick_hash_file(&file_path, 8192, 64).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_full_hash_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let hash = full_hash_file(&file_path, 1).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_same_content_same_hash() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        let content = "Same content for both files";
        fs::write(&file1, content).unwrap();
        fs::write(&file2, content).unwrap();

        let hash1 = full_hash_file(&file1, 1).unwrap();
        let hash2 = full_hash_file(&file2, 1).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        fs::write(&file1, "Content A").unwrap();
        fs::write(&file2, "Content B").unwrap();

        let hash1 = full_hash_file(&file1, 1).unwrap();
        let hash2 = full_hash_file(&file2, 1).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_quick_hash_equals_full_for_small_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("small.txt");
        fs::write(&file_path, "Small file").unwrap();

        let quick = quick_hash_file(&file_path, 8192, 64).unwrap();
        let full = full_hash_file(&file_path, 1).unwrap();
        assert_eq!(quick, full);
    }
}
