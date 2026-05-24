#!/bin/bash
# Background scanner that processes multiple directories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MIRRORS_DIR="${MIRRORS_DIR:-$HOME/git/host}"
NEWROOT_DIR="${NEWROOT_DIR:-$SCRIPT_DIR/newroot_bg}"
LOG_FILE="${LOG_FILE:-$SCRIPT_DIR/scan.log}"

mkdir -p "$NEWROOT_DIR"

echo "=== Background Scanner Started at $(date) ===" | tee -a "$LOG_FILE"
echo "Mirrors: $MIRRORS_DIR" | tee -a "$LOG_FILE"
echo "Newroot: $NEWROOT_DIR" | tee -a "$LOG_FILE"

DIRECTORIES=(
    "./workload"
    "./zkperf"
    "./test_solana_processing"
    "."
)

for dir in "${DIRECTORIES[@]}"; do
    if [ -d "$dir" ]; then
        dirname=$(basename "$dir")
        echo "" | tee -a "$LOG_FILE"
        echo "=== Scanning $dir ===" | tee -a "$LOG_FILE"
        
        timeout 300 cargo run --bin full-scan -- \
            --scan-root "$dir" \
            --mirrors-dir "$MIRRORS_DIR" \
            --newroot-dir "$NEWROOT_DIR" \
            --max-depth 4 \
            --max-iterations 5 \
            2>&1 | tee -a "$LOG_FILE"
        
        echo "=== Completed $dir at $(date) ===" | tee -a "$LOG_FILE"
    fi
done

echo "" | tee -a "$LOG_FILE"
echo "=== All scans completed at $(date) ===" | tee -a "$LOG_FILE"

echo "=== Mirror count ===" | tee -a "$LOG_FILE"
find "$MIRRORS_DIR" -name "*.git" -type d 2>/dev/null | wc -l | tee -a "$LOG_FILE"

echo "=== Newroot submodules ===" | tee -a "$LOG_FILE"
ls "$NEWROOT_DIR" 2>/dev/null | wc -l | tee -a "$LOG_FILE"