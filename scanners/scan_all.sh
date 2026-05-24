#!/bin/bash
# Unified scanner workflow
# Discovers git repos, scans for CBOR files, trains HMMs, detects anomalies

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Build all scanners
build_scanners() {
    log_info "Building scanners..."
    cd "$SCRIPT_DIR"
    cargo build --release --all
    log_success "Scanners built"
}

# Step 2: Discover git repositories
discover_repos() {
    local root="${1:-$WORKSPACE_DIR}"
    local output="${2:-$WORKSPACE_DIR/repos.json}"
    
    log_info "Discovering git repositories in $root..."
    cd "$SCRIPT_DIR"
    ./target/release/git_scanner discover --root "$root" --output "$output" --recursive
    log_success "Found repositories, saved to $output"
}

# Step 3: Extract constants from CBOR files
extract_constants() {
    local repos_file="${1:-$WORKSPACE_DIR/repos.json}"
    local output_dir="${2:-$WORKSPACE_DIR/analysis}"
    
    log_info "Extracting constants from CBOR files..."
    mkdir -p "$output_dir"
    
    # Read repos from JSON and extract CBOR files
    cd "$SCRIPT_DIR"
    
    # For now, just scan the current workspace for CBOR files
    if [ -f "$repos_file" ]; then
        # Parse repos and scan each
        while IFS= read -r repo_path; do
            if [ -d "$repo_path" ]; then
                log_info "Scanning $repo_path for CBOR files..."
                ./target/release/cbor_scanner extract --input "$repo_path" --output "$output_dir/$(basename "$repo_path").cbor.json"
            fi
        done < <(jq -r '.[].path' "$repos_file" 2>/dev/null || echo "$WORKSPACE_DIR")
    else
        # Direct scan
        ./target/release/cbor_scanner extract --input "$WORKSPACE_DIR" --output "$output_dir/constants.json"
    fi
    
    log_success "Constants extracted to $output_dir"
}

# Step 4: Train HMM on CBOR files
train_hmm() {
    local input_dir="${1:-$WORKSPACE_DIR}"
    local output="${2:-$WORKSPACE_DIR/cbor_hmm.json}"
    local states="${3:-8}"
    
    log_info "Training HMM on CBOR files..."
    cd "$SCRIPT_DIR"
    ./target/release/cbor_scanner train --input "$input_dir" --output "$output" --states "$states"
    log_success "HMM trained, saved to $output"
}

# Step 5: Scan with HMM for anomalies
scan_anomalies() {
    local input_dir="${1:-$WORKSPACE_DIR}"
    local model="${2:-$WORKSPACE_DIR/cbor_hmm.json}"
    local output="${3:-$WORKSPACE_DIR/anomalies}"
    
    log_info "Scanning for anomalies..."
    cd "$SCRIPT_DIR"
    ./target/release/cbor_scanner scan --input "$input_dir" --model "$model" --output "$output"
    log_success "Anomaly scan complete, results in $output"
}

# Full workflow
full_workflow() {
    local root="${1:-$WORKSPACE_DIR}"
    local output_dir="${2:-$WORKSPACE_DIR/scanner_output}"
    
    log_info "Starting full scanner workflow on $root..."
    
    mkdir -p "$output_dir"
    
    build_scanners
    discover_repos "$root" "$output_dir/repos.json"
    extract_constants "$output_dir/repos.json" "$output_dir"
    train_hmm "$root" "$output_dir/cbor_hmm.json" 8
    scan_anomalies "$root" "$output_dir/cbor_hmm.json" "$output_dir/anomalies"
    
    log_success "Full workflow complete! Results in $output_dir"
}

# Show help
show_help() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  build              Build all scanners"
    echo "  discover [root]    Discover git repositories"
    echo "  extract [repos]    Extract constants from CBOR files"
    echo "  train [dir]        Train HMM on CBOR files"
    echo "  scan [dir]         Scan for anomalies with HMM"
    echo "  full [root]        Run full workflow"
    echo ""
    echo "Options:"
    echo "  --output, -o      Output directory (default: ./scanner_output)"
    echo "  --states, -s      Number of HMM states (default: 8)"
    echo "  --help, -h        Show this help"
}

# Parse arguments
if [ $# -eq 0 ]; then
    show_help
    exit 1
fi

COMMAND="$1"
shift

case "$COMMAND" in
    build)
        build_scanners
        ;;
    discover)
        discover_repos "${1:-$WORKSPACE_DIR}" "${2:-$WORKSPACE_DIR/repos.json}}"
        ;;
    extract)
        extract_constants "${1:-$WORKSPACE_DIR/repos.json}" "${2:-$WORKSPACE_DIR/analysis}"
        ;;
    train)
        train_hmm "${1:-$WORKSPACE_DIR}" "${2:-$WORKSPACE_DIR/cbor_hmm.json}" "${3:-8}"
        ;;
    scan)
        scan_anomalies "${1:-$WORKSPACE_DIR}" "${2:-$WORKSPACE_DIR/cbor_hmm.json}" "${3:-$WORKSPACE_DIR/anomalies}"
        ;;
    full)
        full_workflow "${1:-$WORKSPACE_DIR}" "${2:-$WORKSPACE_DIR/scanner_output}"
        ;;
    *)
        show_help
        exit 1
        ;;
esac
