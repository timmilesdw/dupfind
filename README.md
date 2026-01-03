# dupfind

[![CI](https://github.com/timmilesdw/dupfind/actions/workflows/ci.yml/badge.svg)](https://github.com/timmilesdw/dupfind/actions/workflows/ci.yml)

Fast duplicate file finder in Rust. Uses two-phase hashing (quick hash -> full hash) and parallel processing.

> **Note:** This is a learning project. For critical/production use, consider mature tools like [fclones](https://github.com/pkolaczk/fclones) or [jdupes](https://github.com/jbruchon/jdupes).

## Installation

```bash
cargo build --release
```

## Usage

```bash
dupfind /path/to/scan
dupfind -o results.json ~/Documents
dupfind -L --min-size 1024 /data    # follow symlinks, skip small files
dupfind -i logs -i tmp /project     # ignore additional directories
```

### Options

```
-L, --follow-links     Follow symbolic links
-l, --log-level        Log level (off, error, warn, info, debug, trace)
-o, --output-json      Save results to JSON file
-i, --ignore           Additional directories to ignore (repeatable)
--no-ignore            Don't ignore common dirs (.git, node_modules, etc.)
--min-size             Skip files smaller than N bytes
--threads              Thread count (0 = auto)
--quick-hash-size      Quick hash sample size in bytes (default: 8192)
--quick-buffer-size    Quick hash buffer in KB (default: 64)
--full-buffer-size     Full hash buffer in MB (default: 1)
```

## Benchmarks

> ⚠️ These benchmarks were generated with AI assistance on synthetic data. Take them with a grain of salt and run your own tests.

Compared against [fclones](https://github.com/pkolaczk/fclones) (Rust) and [fdupes](https://github.com/adrianlopezroche/fdupes) (C).

### Small files (100 files, 5MB total)

| Tool | Time | Notes |
|------|------|-------|
| **dupfind** | **5.9ms** | |
| fdupes | 21ms | 3.6x slower |
| fclones | 32ms | 5.5x slower |

### Large files (40 files, 400MB total)

| Tool | Time | Notes |
|------|------|-------|
| **fclones** | **43ms** | Better I/O optimization |
| dupfind | 49ms | 1.14x slower |
| fdupes | 622ms | Single-threaded |

### Why dupfind is faster on small files

dupfind has minimal startup overhead, it does only:
1. Group by size
2. Quick hash (first 8KB)
3. Full hash (only for quick hash collisions)

fclones performs additional analysis (hardlink detection, path grouping, prefix/suffix matching) which adds overhead but provides more features.

### Run benchmarks yourself

```bash
./bench/run.sh --quick           # fast test
./bench/run.sh                   # full suite
./bench/run.sh --real ~/Photos   # on real data
```

## Built with

- [rayon](https://github.com/rayon-rs/rayon) - parallel iterators
- [blake3](https://github.com/BLAKE3-team/BLAKE3) - fast cryptographic hashing
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [walkdir](https://github.com/BurntSushi/walkdir) - recursive directory traversal
- [indicatif](https://github.com/console-rs/indicatif) - progress bars

## License

MIT
