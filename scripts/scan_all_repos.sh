#!/usr/bin/env bash
# Run graph analysis on all Rust repos in ~/projects/
set -euo pipefail

PROJECTS_DIR="${1:-/home/mdupont/projects}"
OUTPUT_BASE="${2:-analysis/all-repos}"
GRAPH_BIN="${3:-target/debug/graph}"

# Check graph binary exists
if [ ! -x "$GRAPH_BIN" ]; then
  echo "Building graph binary..."
  cargo build --bin graph 2>&1 | tail -3
  GRAPH_BIN="target/debug/graph"
fi

mkdir -p "$OUTPUT_BASE"
SUMMARY="$OUTPUT_BASE/_index.csv"
echo "repo,nodes,edges,scc,status" > "$SUMMARY"

TOTAL=0
RUST_COUNT=0
SKIPPED=0
FAILED=0

echo "=== Scanning repos in $PROJECTS_DIR ==="
for repo in "$PROJECTS_DIR"/*/; do
  name=$(basename "$repo")
  TOTAL=$((TOTAL + 1))
  
  # Skip if no Cargo.toml
  if [ ! -f "$repo/Cargo.toml" ]; then
    SKIPPED=$((SKIPPED + 1))
    echo "[SKIP] $name (no Cargo.toml)"
    echo "$name,0,0,0,SKIP" >> "$SUMMARY"
    continue
  fi
  
  RUST_COUNT=$((RUST_COUNT + 1))
  OUTDIR="$OUTPUT_BASE/$name"
  mkdir -p "$OUTDIR"
  
  echo ""
  echo "======================================================================"
  echo "[$RUST_COUNT/$TOTAL] Analyzing $name ..."
  echo "======================================================================"
  
  # Run with timeout (10 min per repo)
  if timeout 600 "$GRAPH_BIN" build -w "$repo" -o "$OUTDIR/graph" --include-dev --include-build > "$OUTDIR/build.log" 2>&1; then
    if [ -f "$OUTDIR/graph/graph.json" ]; then
      # Extract summary
      NODES=$(jq -r '.nodes | length' "$OUTDIR/graph/graph.json" 2>/dev/null || echo "?")
      EDGES=$(jq -r '.edges | length' "$OUTDIR/graph/graph.json" 2>/dev/null || echo "?")
      
      # Run analyze
      "$GRAPH_BIN" analyze -i "$OUTDIR/graph/graph.json" -o "$OUTDIR" > /dev/null 2>&1 || true
      "$GRAPH_BIN" visualize -i "$OUTDIR/graph/graph.json" -O "$OUTDIR/graph.dot" > /dev/null 2>&1 || true
      "$GRAPH_BIN" partition -i "$OUTDIR/graph/graph.json" -o "$OUTDIR/partitions" > /dev/null 2>&1 || true
      
      # Get SCC count from analysis
      SCC=$(jq -r '.scc_count // 0' "$OUTDIR/analysis.json" 2>/dev/null || echo "0")
      
      echo "$name,$NODES,$EDGES,$SCC,DONE" >> "$SUMMARY"
      echo "[DONE] $name: $NODES nodes, $EDGES edges"
    else
      echo "$name,0,0,0,NO_GRAPH" >> "$SUMMARY"
      echo "[WARN] $name: build completed but no graph.json"
    fi
  else
    echo "$name,0,0,0,TIMEOUT" >> "$SUMMARY"
    echo "[TIMEOUT] $name"
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "======================================================================"
echo "SUMMARY"
echo "======================================================================"
echo "Total dirs scanned:  $TOTAL"
echo "Rust projects:       $RUST_COUNT"
echo "Skipped (no Rust):   $SKIPPED"
echo "Timed out / failed:  $FAILED"
echo "Results in:          $OUTPUT_BASE"
echo ""
echo "=== Results table ==="
column -t -s',' "$SUMMARY" 2>/dev/null || cat "$SUMMARY"

# Print non-Rust repos separately
echo ""
echo "=== Non-Rust dirs (skipped) ==="
for repo in "$PROJECTS_DIR"/*/; do
  name=$(basename "$repo")
  [ ! -f "$repo/Cargo.toml" ] && echo "  $name"
done
