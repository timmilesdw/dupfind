mod args;
mod hasher;
mod output;
mod scanner;
mod statistics;
mod utils;

use anyhow::Context;
use args::Args;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use output::{print_results, save_results_json};
use scanner::{group_by_size, scan_files};
use statistics::calculate_statistics;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Instant;
use utils::{INTERRUPTED, validate_path};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::Relaxed);
        eprintln!("\nInterrupted by user, cleaning up...");
    })
    .context("Failed to set signal handler")?;

    env_logger::builder()
        .filter_level(args.log_level)
        .format_timestamp_secs()
        .init();

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    let start_time = Instant::now();
    let dir = Path::new(&args.path);
    validate_path(dir)?;

    info!("Starting duplicate file scan in {}", dir.display());
    info!(
        "Configuration: quick_hash={}B, quick_buf={}KB, full_buf={}MB",
        args.quick_hash_size, args.quick_buffer_size, args.full_buffer_size
    );

    let scan_progress = ProgressBar::new_spinner();
    scan_progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    scan_progress.set_message("Scanning files...");

    let files = scan_files(
        dir,
        args.follow_links,
        args.min_size,
        args.no_ignore,
        &args.ignore,
        &scan_progress,
    )?;
    let msg = format!("Found {} files", files.len());
    scan_progress.finish_with_message(msg);

    if files.is_empty() {
        info!("No files found to process");
        return Ok(());
    }

    let group_progress = ProgressBar::new(files.len() as u64);
    group_progress.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap(),
    );
    group_progress.set_message("Grouping by size...");

    let groups = group_by_size(&files, &group_progress)?;
    let num_size_groups = groups.len();
    let msg = format!("Found {} size groups", num_size_groups);
    group_progress.finish_with_message(msg);

    if groups.is_empty() {
        info!("No potential duplicates found");
        return Ok(());
    }

    let total_to_hash: usize = groups.values().map(|files| files.len()).sum();
    let hash_progress = ProgressBar::new(total_to_hash as u64);
    hash_progress.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.green/yellow} {pos:>7}/{len:7} {percent:>3}% {msg}")
            .unwrap(),
    );
    hash_progress.set_message("Computing hashes...");

    let hashes = hasher::compute_hashes(
        groups,
        args.quick_hash_size,
        args.quick_buffer_size,
        args.full_buffer_size,
        &hash_progress,
    )?;

    hash_progress.finish_with_message("Hash computation completed");

    let stats = calculate_statistics(&hashes, files.len(), num_size_groups)?;
    let duration = start_time.elapsed().as_secs_f64();

    print_results(&stats, &hashes)?;

    if let Some(json_path) = args.output_json {
        save_results_json(&json_path, &stats, &hashes, duration)?;
        info!("Results saved to {}", json_path.display());
    }

    info!(
        "Scan completed in {:.2}s: {} duplicate groups, {} files, {} wasted",
        duration,
        stats.total_duplicate_groups,
        stats.total_duplicate_files,
        humansize::format_size(stats.total_wasted_space, humansize::DECIMAL)
    );

    Ok(())
}
