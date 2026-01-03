use anyhow::{Context, Result};
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use supports_hyperlinks::Stream;

use crate::statistics::{DuplicateGroup, ScanResults, ScanStatistics};

fn format_path(path: &Path) -> String {
    if supports_hyperlinks::on(Stream::Stdout) {
        let display = path.display();
        let uri = format!("file://{}", path.display());
        format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", uri, display)
    } else {
        path.display().to_string()
    }
}

pub fn print_results(stats: &ScanStatistics, hashes: &HashMap<String, Vec<PathBuf>>) -> Result<()> {
    if stats.total_duplicate_groups == 0 {
        println!("{}", "No duplicates found.".green());
        return Ok(());
    }

    println!(
        "\n{} {} {} ({})",
        "Found".bold(),
        stats.total_duplicate_groups.to_string().yellow().bold(),
        if stats.total_duplicate_groups == 1 {
            "duplicate group"
        } else {
            "duplicate groups"
        },
        format_size(stats.total_wasted_space, DECIMAL).red().bold()
    );
    println!();

    let mut sorted_groups: Vec<_> = hashes
        .iter()
        .filter_map(|(hash, files)| {
            let existing_files: Vec<_> =
                files.iter().filter(|path| path.exists()).cloned().collect();
            if existing_files.len() < 2 {
                return None;
            }
            let size = std::fs::metadata(&existing_files[0])
                .ok()
                .map(|m| m.len())
                .unwrap_or(0);
            Some((hash.clone(), existing_files, size))
        })
        .collect();

    sorted_groups.sort_by(|a, b| {
        let wasted_a = a.1.len() as u64 * a.2;
        let wasted_b = b.1.len() as u64 * b.2;
        wasted_b.cmp(&wasted_a)
    });

    for (idx, (_hash, files, size)) in sorted_groups.iter().enumerate() {
        let wasted = size * (files.len() as u64 - 1);

        println!(
            "{} {} {} {} {}",
            format!("#{}", idx + 1).cyan().bold(),
            "·".dimmed(),
            format_size(*size, DECIMAL).white(),
            "×".dimmed(),
            format!("{} files", files.len()).white(),
        );

        for (i, path) in files.iter().enumerate() {
            let prefix = if i == 0 {
                "  ├".dimmed()
            } else if i == files.len() - 1 {
                "  └".dimmed()
            } else {
                "  │".dimmed()
            };
            println!("{} {}", prefix, format_path(path));
        }

        println!(
            "    {} {}",
            "wasted:".dimmed(),
            format_size(wasted, DECIMAL).red()
        );
        println!();
    }

    Ok(())
}

pub fn save_results_json(
    path: &Path,
    stats: &ScanStatistics,
    hashes: &HashMap<String, Vec<PathBuf>>,
    duration: f64,
) -> Result<()> {
    let groups: Vec<DuplicateGroup> = hashes
        .iter()
        .filter_map(|(hash, files)| {
            let existing_files: Vec<_> = files
                .iter()
                .filter(|p| p.exists())
                .filter_map(|p| p.to_str().map(String::from))
                .collect();

            if existing_files.len() < 2 {
                return None;
            }

            let size = std::fs::metadata(&files[0])
                .ok()
                .map(|m| m.len())
                .unwrap_or(0);

            Some(DuplicateGroup {
                hash: hash.clone(),
                size,
                files: existing_files,
            })
        })
        .collect();

    let results = ScanResults {
        total_files_scanned: stats.total_files_scanned,
        total_size_groups: stats.total_size_groups,
        total_duplicate_groups: stats.total_duplicate_groups,
        total_duplicate_files: stats.total_duplicate_files,
        total_wasted_space: stats.total_wasted_space,
        scan_duration_seconds: duration,
        groups,
    };

    let json =
        serde_json::to_string_pretty(&results).context("Failed to serialize results to JSON")?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("Failed to open output file: {}", path.display()))?;

    file.write_all(json.as_bytes())
        .context("Failed to write JSON output")?;

    Ok(())
}
