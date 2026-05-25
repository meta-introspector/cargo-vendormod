---
name: build-graph
description: Build cargo dependency graphs from Cargo.toml/Cargo.lock using the `graph` binary. Use when the user asks to analyze, scan, or build a dependency graph for a Rust/Cargo project. Triggers on: "build graph", "scan deps", "analyze dependencies", "dependency graph", "cargo graph", "graph build".
---

# Build Graph

Build dependency graphs for Cargo projects using the `graph` binary.

## Prerequisites

- `graph` binary built from the workspace: `cargo build --bin graph`
- Target project directory with `Cargo.toml` + `Cargo.lock`

## Usage

```
./target/debug/graph build -w <project-dir> -o <output-dir> [--include-dev] [--include-build]
```

The binary uses a pure-Rust Cargo.lock parser (falls back automatically when `cargo metadata` is unavailable). No shell commands, no `find`, no Python.

## Fallback Behavior

1. **Primary**: `cargo_metadata::MetadataCommand` — works when `cargo` is in PATH
2. **Fallback**: `analyze_from_lockfile()` — pure-Rust `toml_edit` parser, works in Nix sandbox, containers, minimal PATH environments

## Output

```
<output-dir>/
├── graph.json     # GlobalDependencyGraph (JSON)
├── summary.txt    # Human-readable summary
└── analysis.json  # Metrics (nodes, edges, SCC count, density)
```

## Graph JSON Format

```json
{
  "nodes": [
    {
      "id": "crate:serde:1.0.203",
      "crate_name": "serde",
      "version": "1.0.203",
      "source": "registry+https://github.com/rust-lang/crates.io-index",
      "is_workspace_member": false,
      "edge_count": 0
    }
  ],
  "edges": [
    { "from": "crate:foo:0.1.0", "to": "crate:bar:0.2.0", "edge_type": "dependency" }
  ],
  "scc_count": 0,
  "total_scc_nodes": 0,
  "merged_projects": [],
  "statistics": { "total_nodes": 323, "total_edges": 705 }
}
```

## Integration

- Batch scanning: `for d in projects/*/; do graph build -w "$d" -o "graphs/$(basename $d)"; done`
- Makefile target: `make projects-graphs` (uses PROJECTS_DIR=/home/mdupont/projects)
- Nix: `nix build --impure .#graph-<project-name>`

## Examples

```bash
# Single project
./target/debug/graph build -w /home/mdupont/projects/pastebin -o /tmp/pastebin_graph

# Self-analysis
./target/debug/graph build -w . -o analysis/self_graph --include-dev --include-build

# Nix sandbox (no cargo binary available)
nix build --impure .#graph-pastebin
```

## Troubleshooting

- **"No Cargo.lock found"**: Run `cargo generate-lockfile` first
- **"cargo metadata failed"**: Falls back automatically to lockfile parser
- **"Only 1 node"**: Check that Cargo.toml is valid and Cargo.lock exists
- **Slow discovery**: Run from within the workspace root (not parent dir with 1000 subdirs)
