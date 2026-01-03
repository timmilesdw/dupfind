#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_DIR="/tmp/dupfind_bench"
DUPFIND="$PROJECT_DIR/target/release/dupfind"
RESULTS_FILE="$TEST_DIR/all_results.txt"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

check_deps() {
    local missing=()
    command -v hyperfine >/dev/null || missing+=("hyperfine")
    command -v fdupes >/dev/null || missing+=("fdupes")
    command -v fclones >/dev/null || missing+=("fclones")
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${YELLOW}Missing dependencies: ${missing[*]}${NC}"
        echo "Install with:"
        echo "  brew install hyperfine fdupes"
        echo "  cargo install fclones"
        exit 1
    fi
}

build_release() {
    echo -e "${GREEN}Building dupfind...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release --quiet
}

create_test_data() {
    local size=$1
    local name=$2
    
    echo -e "${GREEN}Creating test data: $name${NC}"
    rm -rf "$TEST_DIR/$name"
    mkdir -p "$TEST_DIR/$name"
    
    case $size in
        small)
            # 100 files, 50 duplicates, 1KB each
            for i in $(seq 1 50); do
                dd if=/dev/urandom of="$TEST_DIR/$name/file_$i.bin" bs=1K count=1 2>/dev/null
                cp "$TEST_DIR/$name/file_$i.bin" "$TEST_DIR/$name/dup_$i.bin"
            done
            ;;
        medium)
            # 500 files, 250 duplicates, 100KB each
            for i in $(seq 1 250); do
                dd if=/dev/urandom of="$TEST_DIR/$name/file_$i.bin" bs=100K count=1 2>/dev/null
                cp "$TEST_DIR/$name/file_$i.bin" "$TEST_DIR/$name/dup_$i.bin"
            done
            ;;
        large)
            # 200 files, 100 duplicates, 1MB each
            for i in $(seq 1 100); do
                dd if=/dev/urandom of="$TEST_DIR/$name/file_$i.bin" bs=1M count=1 2>/dev/null
                cp "$TEST_DIR/$name/file_$i.bin" "$TEST_DIR/$name/dup_$i.bin"
            done
            ;;
        mixed)
            # Mix of sizes
            for i in $(seq 1 50); do
                dd if=/dev/urandom of="$TEST_DIR/$name/small_$i.bin" bs=1K count=1 2>/dev/null
                cp "$TEST_DIR/$name/small_$i.bin" "$TEST_DIR/$name/small_dup_$i.bin"
            done
            for i in $(seq 1 30); do
                dd if=/dev/urandom of="$TEST_DIR/$name/med_$i.bin" bs=100K count=1 2>/dev/null
                cp "$TEST_DIR/$name/med_$i.bin" "$TEST_DIR/$name/med_dup_$i.bin"
            done
            for i in $(seq 1 10); do
                dd if=/dev/urandom of="$TEST_DIR/$name/large_$i.bin" bs=1M count=1 2>/dev/null
                cp "$TEST_DIR/$name/large_$i.bin" "$TEST_DIR/$name/large_dup_$i.bin"
            done
            ;;
    esac
    
    echo "  Files: $(ls "$TEST_DIR/$name" | wc -l | tr -d ' ')"
    echo "  Size: $(du -sh "$TEST_DIR/$name" | cut -f1)"
}

run_benchmark() {
    local name=$1
    local dir="$TEST_DIR/$name"
    
    echo -e "\n${GREEN}=== Benchmark: $name ===${NC}\n"
    
    # Run hyperfine and capture JSON output
    hyperfine \
        --warmup 2 \
        --min-runs 5 \
        --export-json "$TEST_DIR/results_$name.json" \
        "$DUPFIND -l off $dir" \
        "fclones group $dir 2>/dev/null" \
        "fdupes -r $dir"
    
    # Extract times and save to results file
    local dupfind_time=$(jq -r '.results[0].mean * 1000 | floor' "$TEST_DIR/results_$name.json")
    local fclones_time=$(jq -r '.results[1].mean * 1000 | floor' "$TEST_DIR/results_$name.json")
    local fdupes_time=$(jq -r '.results[2].mean * 1000 | floor' "$TEST_DIR/results_$name.json")
    echo "$name|$dupfind_time|$fclones_time|$fdupes_time" >> "$RESULTS_FILE"
}

print_results_table() {
    echo ""
    echo -e "${GREEN}${BOLD}╔═══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}${BOLD}║              BENCHMARK RESULTS (ms)                   ║${NC}"
    echo -e "${GREEN}${BOLD}╠═══════════════════════════════════════════════════════╣${NC}"
    printf "${GREEN}${BOLD}║${NC} ${BOLD}%-15s │ %10s │ %10s │ %10s ${GREEN}${BOLD}║${NC}\n" "Test" "dupfind" "fclones" "fdupes"
    echo -e "${GREEN}${BOLD}╠═══════════════════════════════════════════════════════╣${NC}"
    
    while IFS='|' read -r name dupfind fclones fdupes; do
        # Find winner
        local winner="fdupes"
        local min=$fdupes
        if [ "$dupfind" -le "$min" ]; then winner="dupfind"; min=$dupfind; fi
        if [ "$fclones" -le "$min" ]; then winner="fclones"; fi
        
        # Format with colors for winner
        local d_col="" f_col="" fd_col=""
        [ "$winner" = "dupfind" ] && d_col="${GREEN}${BOLD}"
        [ "$winner" = "fclones" ] && f_col="${CYAN}${BOLD}"
        [ "$winner" = "fdupes" ] && fd_col="${YELLOW}${BOLD}"
        
        printf "${GREEN}${BOLD}║${NC} %-15s │ ${d_col}%10s${NC} │ ${f_col}%10s${NC} │ ${fd_col}%10s${NC} ${GREEN}${BOLD}║${NC}\n" \
            "$name" "$dupfind" "$fclones" "$fdupes"
    done < "$RESULTS_FILE"
    
    echo -e "${GREEN}${BOLD}╚═══════════════════════════════════════════════════════╝${NC}"
    echo ""
}

cleanup() {
    echo -e "\n${YELLOW}Cleaning up test data...${NC}"
    rm -rf "$TEST_DIR"
}

main() {
    echo -e "${GREEN}╔══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     dupfind benchmark suite          ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════╝${NC}\n"
    
    check_deps
    build_release
    
    # Check for jq
    if ! command -v jq >/dev/null; then
        echo -e "${YELLOW}Note: install jq for summary table (brew install jq)${NC}"
    fi
    
    # Init results file
    mkdir -p "$TEST_DIR"
    > "$RESULTS_FILE"
    
    # Create test datasets
    create_test_data small "small_files"
    create_test_data medium "medium_files"
    create_test_data large "large_files"
    create_test_data mixed "mixed_files"
    
    # Run benchmarks
    run_benchmark "small_files"
    run_benchmark "medium_files"
    run_benchmark "large_files"
    run_benchmark "mixed_files"
    
    # Print summary table
    if command -v jq >/dev/null && [ -s "$RESULTS_FILE" ]; then
        print_results_table
    fi
    
    # Optional cleanup
    read -p "Clean up test data? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup
    fi
}

# Allow running specific benchmark
if [ "$1" = "--quick" ]; then
    check_deps
    build_release
    mkdir -p "$TEST_DIR"
    > "$RESULTS_FILE"
    create_test_data small "quick_test"
    run_benchmark "quick_test"
    if command -v jq >/dev/null && [ -s "$RESULTS_FILE" ]; then
        print_results_table
    fi
    rm -rf "$TEST_DIR"
elif [ "$1" = "--real" ] && [ -n "$2" ]; then
    check_deps
    build_release
    echo -e "${GREEN}Benchmarking on real directory: $2${NC}"
    hyperfine \
        --warmup 1 \
        --min-runs 3 \
        --shell=none \
        "$DUPFIND -l off $2" \
        "fclones group $2" \
        "fdupes -r $2"
else
    main
fi

