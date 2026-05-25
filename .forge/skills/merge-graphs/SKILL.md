---
name: merge-graphs
description: Merge per-project dependency graphs into a single global graph with topological sort, upgrade planning, bare repo mirrors, and per-crate Nix flakes. Use when the user asks to combine multiple graphs, create a global dependency view, find version mismatches, generate upgrade plans, or create build orders. Triggers on: "merge graphs", "global graph", "upgrade plan", "build order", "version mismatch", "combine dependencies".
---

# Merge Graphs

Merge multiple per-project dependency graphs into a unified global graph
with analysis outputs.

## Prerequisites

- `graph` binary built from workspace
- Multiple project graphs in `projects_graphs/<name>/graph/graph.json` format
- Use `build-graph` skill first to generate per-project graphs

## Usage

```
./target/debug/graph merge -i <input-dir> -o <output-dir>
```

The input directory should contain subdirectories, each with a `graph/graph.json` file.

## Output

```
<output-dir>/
├── global_graph.json          # Merged graph (all nodes + edges)
├── build_order.json           # Topological sort of all crates
├── upgrade_plan.json          # Version-mismatch candidates
├── projects.json              # Index of merged projects
├── mirrors.json               # Git source URLs for mirroring
├── cargo_mirror_config.toml   # Replace remote URLs with local bare repos
├── bare_repos/                # Empty bare git repos (one per unique git source)
├── flakes/                    # Minimal flake.nix per unique crate
│   └── <crate_name>/
│       └── flake.nix
└── summary.txt                # Human-readable summary
```

### build_order.json Format

```json
{
  "total_crates": 4066,
  "order": [
    { "crate": "hashbrown", "version": "0.14.3", "position": 1 },
    { "crate": "fnv", "version": "1.0.7", "position": 2 },
    ...
  ],
  "scc_count": 0
}
```

### upgrade_plan.json Format

```json
{
  "total_candidates": 674,
  "plans": [
    {
      "crate": "windows_x86_64_gnu",
      "oldest": "0.42.2",
      "newest": "0.53.1",
      "project_versions": {
        "project-a": "0.42.2",
        "project-b": "0.53.1"
      }
    }
  ]
}
```

## Bare Repo Setup

The merge command creates empty bare git repos for each unique git dependency source.
To populate them with actual content:

```bash
cd <output-dir>/bare_repos
for repo in *.git; do
  url=$(cat "../cargo_mirror_config.toml" | grep "$repo" | head -1)
  # Clone from remote into bare repo
  git clone --bare "$url" "$repo"
done
```

## Crate Flake Contents

Each `flakes/<crate>/flake.nix` contains:
- Inputs: nixpkgs, flake-utils, rust-overlay
- `buildRustPackage` default package template
- `mkShell` devShell template

## Integration

- Makefile: `make global-graph`, `make build-order`, `make upgrade-plan`
- After scanning new projects: `graph merge -i projects_graphs -o global_graph`

## Examples

```bash
# Basic merge
./target/debug/graph merge -i projects_graphs -o global_graph

# After adding a new project scan (e.g., ragit)
./target/debug/graph merge -i projects_graphs -o global_graph

# Show build order (top 5)
python3 -c "import json; d=json.load(open('global_graph/build_order.json')); [print(o['crate']) for o in d['order'][:5]]"
```
