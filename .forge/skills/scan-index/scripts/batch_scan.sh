#!/usr/bin/env bash
# Batch scan: scan multiple projects from cargos.txt by name pattern
# Usage: ./batch_scan.sh <pattern>
# Example: ./batch_scan.sh "ragit|streamofrandom|neo"
set -euo pipefail
PATTERN="${1:?Usage: $0 <regex-pattern>}"
INDEX="$HOME/2026/05/25/cargos.txt"
GRAPH_BIN="/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/target/debug/graph"
OUTDIR="$(dirname "$GRAPH_BIN")/../projects_graphs"

echo "Scanning projects matching: $PATTERN"
python3 -c "
import os, subprocess, json, sys

with open('$INDEX') as f:
    lines = [l.strip() for l in f if l.strip()]

import re
pat = re.compile('$PATTERN', re.IGNORECASE)

found = set()
for l in lines:
    if pat.search(l) and l.endswith('Cargo.toml'):
        d = os.path.dirname(l)
        if os.path.exists(d + '/Cargo.lock'):
            found.add(d)

print(f'Found {len(found)} workspace roots...')
for d in sorted(found):
    name = os.path.basename(d)
    out = f'$OUTDIR/{name}/graph'
    if os.path.exists(f'{out}/graph.json'):
        print(f'  SKIP {name} (already scanned)')
        continue
    os.makedirs(out, exist_ok=True)
    print(f'  SCAN {name}...', end=' ', flush=True)
    result = subprocess.run(
        ['$GRAPH_BIN', 'build', '-w', d, '-o', out, '--include-dev', '--include-build'],
        capture_output=True, text=True, timeout=120
    )
    if result.returncode == 0 and os.path.exists(f'{out}/graph.json'):
        g = json.load(open(f'{out}/graph.json'))
        print(f'{len(g[\"nodes\"])} nodes, {len(g[\"edges\"])} edges')
    else:
        err = result.stderr[-80:].replace(chr(10), ' ')
        print(f'FAILED: {err}')
"
