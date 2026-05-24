#!/bin/bash
# Monitor the background scan

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${1:-$SCRIPT_DIR/scan.log}"
MIRRORS_DIR="${MIRRORS_DIR:-$HOME/git/host}"
NEWROOT_DIR="${NEWROOT_DIR:-$SCRIPT_DIR/newroot_bg}"

echo "=== Scan Monitor ==="
echo "Log: $LOG_FILE"
echo ""

if [ ! -f "$LOG_FILE" ]; then
    echo "No log file found yet"
    exit 1
fi

echo "=== Last 30 lines of log ==="
tail -30 "$LOG_FILE"

echo ""
echo "=== Current counts ==="
echo "Mirrors: $(find "$MIRRORS_DIR" -name "*.git" -type d 2>/dev/null | wc -l)"
echo "Newroot submodules: $(ls "$NEWROOT_DIR" 2>/dev/null | wc -l)"

echo ""
echo "=== Running processes ==="
ps aux | grep -E "(full-scan|cargo)" | grep -v grep || echo "No scan running"