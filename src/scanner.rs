use anyhow::{Result, bail};
use indicatif::ProgressBar;
use log::warn;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use walkdir::WalkDir;

use crate::utils::INTERRUPTED;

const DEFAULT_IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    "target",
    "build",
    "dist",
    ".idea",
    ".vscode",
    ".cache",
    ".npm",
    ".cargo",
    "vendor",
    ".bundle",
    "Pods",
    ".gradle",
    ".m2",
    "bower_components",
];

pub fn scan_files(
    dir: &Path,
    follow_links: bool,
    min_size: u64,
    no_ignore: bool,
    extra_ignore: &[String],
    include_hidden: bool,
    progress: &ProgressBar,
) -> Result<Vec<walkdir::DirEntry>> {
    let ignored: HashSet<&str> = if no_ignore {
        HashSet::new()
    } else {
        DEFAULT_IGNORED_DIRS
            .iter()
            .copied()
            .chain(extra_ignore.iter().map(|s| s.as_str()))
            .collect()
    };

    let mut walker = WalkDir::new(dir);
    if !follow_links {
        walker = walker.follow_links(false);
    }

    let mut files = Vec::new();
    let mut scanned = 0u64;

    let iter = walker.into_iter().filter_entry(|e| {
        if let Some(name) = e.file_name().to_str() {
            if !include_hidden && name.starts_with('.') && name != "." {
                return false;
            }
            if e.file_type().is_dir() && ignored.contains(name) {
                return false;
            }
        }
        true
    });

    for entry in iter {
        if INTERRUPTED.load(Ordering::Relaxed) {
            bail!("Scan interrupted by user");
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Error reading directory entry: {}", e);
                continue;
            }
        };

        if !follow_links && entry.path().is_symlink() {
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("Cannot read metadata for {}: {}", entry.path().display(), e);
                continue;
            }
        };

        let size = metadata.len();
        if size < min_size {
            continue;
        }

        files.push(entry);
        scanned += 1;
        if scanned.is_multiple_of(1000) {
            let msg = format!("Scanned {} files...", scanned);
            progress.set_message(msg);
        }
    }

    Ok(files)
}

pub fn group_by_size(
    files: &[walkdir::DirEntry],
    progress: &ProgressBar,
) -> Result<HashMap<u64, Vec<std::path::PathBuf>>> {
    let processed = Arc::new(AtomicU64::new(0));
    let total = files.len() as u64;

    let groups: HashMap<u64, Vec<std::path::PathBuf>> = files
        .par_iter()
        .filter_map(|file| {
            if INTERRUPTED.load(Ordering::Relaxed) {
                return None;
            }

            let metadata = file.metadata().ok()?;
            let size = metadata.len();
            if size == 0 {
                return None;
            }

            let current = processed.fetch_add(1, Ordering::Relaxed);
            if current.is_multiple_of(1000) {
                progress.set_position(current.min(total));
            }

            Some((size, file.path().to_path_buf()))
        })
        .fold(
            HashMap::<u64, Vec<std::path::PathBuf>>::new,
            |mut acc, (size, path)| {
                acc.entry(size).or_default().push(path);
                acc
            },
        )
        .reduce(HashMap::<u64, Vec<std::path::PathBuf>>::new, |mut a, b| {
            for (size, paths) in b {
                a.entry(size).or_default().extend(paths);
            }
            a
        });

    progress.set_position(total);
    let mut groups = groups;
    groups.retain(|_, files| files.len() > 1);
    Ok(groups)
}
