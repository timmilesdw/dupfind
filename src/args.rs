use clap::Parser;
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Fast parallel duplicate file finder",
    long_about = "Fast, parallel duplicate file finder with progress tracking"
)]
pub struct Args {
    /// Directory to search for duplicates
    pub path: String,

    /// Log level (off, error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    pub log_level: LevelFilter,

    /// Follow symbolic links
    #[arg(short = 'L', long)]
    pub follow_links: bool,

    /// Don't ignore common directories (.git, node_modules, etc.)
    #[arg(long)]
    pub no_ignore: bool,

    /// Additional directories to ignore (can be used multiple times)
    #[arg(short, long = "ignore", value_name = "DIR")]
    pub ignore: Vec<String>,

    /// Include hidden files and directories (starting with '.')
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Quick hash sample size in bytes
    #[arg(long, default_value = "8192")]
    pub quick_hash_size: usize,

    /// Quick hash buffer size in KB
    #[arg(long, default_value = "64")]
    pub quick_buffer_size: usize,

    /// Full hash buffer size in MB
    #[arg(long, default_value = "1")]
    pub full_buffer_size: usize,

    /// Output results to JSON file
    #[arg(short, long)]
    pub output_json: Option<PathBuf>,

    /// Skip files smaller than this size in bytes
    #[arg(long, default_value = "0")]
    pub min_size: u64,

    /// Maximum number of threads (0 = auto)
    #[arg(long, default_value = "0")]
    pub threads: usize,
}
