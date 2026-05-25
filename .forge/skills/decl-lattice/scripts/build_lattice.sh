#!/usr/bin/env bash
# One-click: split a project's decls and build the lattice
# Usage: ./build_lattice.sh <project-name> <source-dir>
set -euo pipefail
NAME="${1:?Usage: $0 <project-name> <source-dir>}"
SRC="${2:?$0 <project-name> <source-dir>}"
DS="/mnt/data1/time-2026/05-may/15/forgecode-decl-splitter/tools/decl_splitter/target/debug/decl-splitter"
SCRIPTS="/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/scripts"

echo "=== Phase 1: Split declarations ==="
DECLS_DIR="/tmp/decls_$NAME"
find "$SRC" -name '*.rs' -not -path '*/target/*' -not -path '*/vendor/*' | \
  xargs -P4 -I{} "$DS" -i {} -o "$DECLS_DIR/{}" 2>&1 | tail -3
DECL_COUNT=$(find "$DECLS_DIR" -name '*.rs' ! -name '_decl*' -type f 2>/dev/null | wc -l)
echo "Declarations: $DECL_COUNT"

echo "=== Phase 2: Build decl graph ==="
python3 "$SCRIPTS/build_decl_graph.py" --input "$DECLS_DIR" --output "/tmp/decl_graph_$NAME"
echo "Graph: $(python3 -c "import json; g=json.load(open('/tmp/decl_graph_$NAME/decl_graph.json')); print(f'{len(g[\"nodes\"])} nodes, {len(g[\"edges\"])} edges')")"

echo "=== Phase 3: Generate lattice crates ==="
LATTICE_DIR="decl_lattice/extra/$NAME"
python3 "$SCRIPTS/gen_decl_lattice.py" --graph "/tmp/decl_graph_$NAME/decl_graph.json" --output "$LATTICE_DIR"
echo "Lattice: $(ls "$LATTICE_DIR/crates" 2>/dev/null | wc -l) crate dirs"

echo "Done. Add 'crates/$NAME/*' to decl_lattice/Cargo.toml workspace members"
