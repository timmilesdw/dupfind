use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResults {
    pub total_files_scanned: usize,
    pub total_size_groups: usize,
    pub total_duplicate_groups: usize,
    pub total_duplicate_files: usize,
    pub total_wasted_space: u64,
    pub scan_duration_seconds: f64,
    pub groups: Vec<DuplicateGroup>,
}

pub struct ScanStatistics {
    pub total_files_scanned: usize,
    pub total_size_groups: usize,
    pub total_duplicate_groups: usize,
    pub total_duplicate_files: usize,
    pub total_wasted_space: u64,
}

pub fn calculate_statistics(
    hashes: &HashMap<String, Vec<PathBuf>>,
    total_files_scanned: usize,
    total_size_groups: usize,
) -> Result<ScanStatistics> {
    let total_duplicate_groups = hashes.len();
    let total_duplicate_files: usize = hashes.values().map(|files| files.len()).sum();

    let total_wasted_space = hashes
        .par_iter()
        .filter_map(|(_, files)| {
            let first_file = files.first()?;
            let size = std::fs::metadata(first_file).ok()?.len();
            let wasted = size * (files.len() as u64 - 1);
            Some(wasted)
        })
        .sum::<u64>();

    Ok(ScanStatistics {
        total_files_scanned,
        total_size_groups,
        total_duplicate_groups,
        total_duplicate_files,
        total_wasted_space,
    })
}
