#!/usr/bin/env bash
# One-click scan: finds workspace root from cargos.txt and builds graph
# Usage: ./scan_one.sh <project-name>
set -euo pipefail
NAME="${1:?Usage: $0 <project-name>}"
INDEX="$HOME/2026/05/25/cargos.txt"
GRAPH_BIN="/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/target/debug/graph"

# Find workspace root
ROOT=$(python3 -c "
import os
with open('$INDEX') as f:
    for l in f:
        l=l.strip()
        if '$NAME' in l.lower() and l.endswith('Cargo.toml'):
            d=os.path.dirname(l)
            if os.path.exists(d + '/Cargo.lock'):
                print(d)
                break
")
if [ -z "$ROOT" ]; then
    echo "No Cargo.toml+Cargo.lock found for '$NAME' in index"
    exit 1
fi
echo "Found: $ROOT"

# Build graph
OUT="projects_graphs/$NAME/graph"
mkdir -p "$OUT"
echo "Building graph..."
"$GRAPH_BIN" build -w "$ROOT" -o "$OUT" --include-dev --include-build 2>&1 | tail -5

# Report
python3 -c "
import json
with open('$OUT/graph.json') as f:
    g=json.load(f)
print(f'Graph: {len(g[\"nodes\"])} nodes, {len(g[\"edges\"])} edges')
ws=[n['crate_name'] for n in g['nodes'] if n.get('is_workspace_member')]
print(f'Workspace members: {len(ws)}')
"
echo "Done. Run: make global-graph  (to merge into global graph)"
