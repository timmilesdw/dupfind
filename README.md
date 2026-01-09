# dupfind

[![CI](https://github.com/timmilesdw/dupfind/actions/workflows/ci.yml/badge.svg)](https://github.com/timmilesdw/dupfind/actions/workflows/ci.yml) ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

Fast duplicate file finder in Rust. Uses two-phase hashing (quick hash -> full hash) and parallel processing.

> **Note:** This is a learning project. For critical/production use, consider mature tools like [fclones](https://github.com/pkolaczk/fclones) or [jdupes](https://codeberg.org/jbruchon/jdupes).

## Installation

### Download binary

Download the latest release from [Releases](https://github.com/timmilesdw/dupfind/releases):

| Platform | File |
|----------|------|
| Linux x64 | `dupfind-*-linux-x64.tar.gz` |
| Linux x64 (static) | `dupfind-*-linux-x64-musl.tar.gz` |
| Linux ARM64 | `dupfind-*-linux-arm64.tar.gz` |
| macOS Intel | `dupfind-*-macos-x64.tar.gz` |
| macOS Apple Silicon | `dupfind-*-macos-arm64.tar.gz` |
| Windows x64 | `dupfind-*-windows-x64.zip` |

```bash
# Example for Linux/macOS
tar -xzf dupfind-*.tar.gz
sudo mv dupfind /usr/local/bin/
```

### Build from source

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
-H, --hidden           Include hidden files and system directories
-l, --log-level        Log level (off, error, warn, info, debug, trace)
-o, --output-json      Save results to JSON file
-i, --ignore           Additional directories to ignore (repeatable)
--min-size             Skip files smaller than N bytes
--threads              Thread count (0 = auto)
```

### What's ignored by default

- **Dotfiles**: files/directories starting with `.` (`.git`, `.cache`, `.Trash`)
- **System directories**:
  - macOS: `~/Library` and others with BSD `UF_HIDDEN` flag
  - Windows: files with `HIDDEN` or `SYSTEM` attributes

Use `-H/--hidden` to include hidden files.

## Benchmarks

> ⚠️ These benchmarks were generated with AI assistance on synthetic data. Take them with a grain of salt and run your own tests.

Compared against [fclones](https://github.com/pkolaczk/fclones) (Rust) and [fdupes](https://github.com/adrianlopezroche/fdupes) (C). Apple M3, macOS.

| Test | dupfind | fclones | fdupes |
|------|---------|---------|--------|
| small (100 files, 400KB) | ~5ms | 31ms | ~5ms |
| medium (500 files, 50MB) | **19ms** | 42ms | 98ms |
| large (200 files, 200MB) | **32ms** | 41ms | 345ms |
| mixed | **9ms** | 32ms | 46ms |

On small files dupfind and fdupes are roughly equal (results vary between runs). On medium/large files dupfind wins thanks to parallel processing.

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
