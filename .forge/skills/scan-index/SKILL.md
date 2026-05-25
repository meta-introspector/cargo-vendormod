---
name: scan-index
description: Discover and scan Rust projects from the 121K-entry cargos.txt index, locating Cargo.toml + Cargo.lock pairs with [workspace] declarations, then building dependency graphs. Use when the user asks to find new projects to analyze, scan from the index, discover workspace roots, or expand coverage beyond the current 38 projects. Triggers on: "scan index", "find projects", "cargos.txt", "discover workspace", "scan workspace", "index scan", "find new crates".
---

# Scan Index

Discover and scan Rust projects from the `/home/mdupont/2026/05/25/cargos.txt` index
(121K Cargo.toml paths).

## The Index

`~/2026/05/25/cargos.txt` contains paths (relative or absolute) to every Cargo.toml
on the system. ~121K entries. Key source directories:

| Source | Count | Description |
|--------|-------|-------------|
| `/mnt/data1/nix/` | 50K | Nix store builds |
| `/mnt/data1/time-2026/` | 20K | Time-stamped worktrees |
| `/mnt/data1/cargo/` | 7K | Crate2nix built crates |
| `/mnt/data1/git/` | 1.5K | Bare git repos |
| `/mnt/data1/introspector/` | 1K | Meta-introspector projects |
| `/home/mdupont/projects/` | 43 | Our currently analyzed set |
| Other | ~41K | Remaining |

## Quick Find (single project)

```bash
# Use the Makefile target
make scan-index N=<project-name>

# This searches cargos.txt, finds matching Cargo.tomls with Cargo.lock,
# and prints the workspace root + graph build command
```

## Batch Discovery (workspace roots)

```bash
# Find all Cargo.tomls with [workspace] markers (scans first 5000 entries)
make index-workspace
```

Output: `projects_graphs/workspace_roots.json`

## Scan a Project into the Graph

```bash
# Once you find a workspace root:
graph build -w <workspace-root> -o projects_graphs/<name>/graph \
  --include-dev --include-build

# Then re-merge the global graph:
graph merge -i projects_graphs -o global_graph
```

## Known Unscanned Projects (from cargos.txt)

| Project | Appears | Status |
|---------|---------|--------|
| ragit | 1,502 Cargo.tomls | ✅ Scanned (457 nodes) |
| streamofrandom | 36 Cargo.tomls | ✅ Scanned (11 sub-projects) |
| arti-tor-rs | ~800 Cargo.tomls | ✅ Scanned (842 nodes) |
| meta-introspector | 2,224 Cargo.tomls | ❌ Not scanned |
| lattice-introspector | 1,989 Cargo.tomls | ❌ Not scanned |
| neo | 5 Cargo.tomls | ❌ Not a Rust workspace |
| Remaining | ~74K workspace dirs | ❌ Not scanned |

## Decl-Splitter Integration

```bash
# After scanning a project, split its declarations
DS=/mnt/data1/time-2026/05-may/15/forgecode-decl-splitter/tools/decl_splitter/target/debug/decl-splitter
find <workspace-root> -name '*.rs' -not -path '*/vendor/*' -not -path '*/target/*' \
  | xargs -P4 -I{} $DS -i {} -o /tmp/decls_<name>/{}
```

## Makefile Automation

```makefile
# Add new project to periodic scan:
# 1. Find workspace root: make scan-index N=<name>
# 2. Build graph:         make projects-graphs (after adding source to PROJECTS_DIR)
# 3. Merge:               make global-graph
# 4. Analyze:             make build-order, make upgrade-plan
```

## Full Automation Script

```bash
# For a new project name found in cargos.txt:
NAME=ragit
ROOT=$(python3 -c "
with open('/home/mdupont/2026/05/25/cargos.txt') as f:
    for l in f:
        l=l.strip()
        if '$NAME' in l.lower() and l.endswith('Cargo.toml'):
            import os; d=os.path.dirname(l)
            if os.path.exists(d + '/Cargo.lock'):
                print(d); break
")
./target/debug/graph build -w "$ROOT" -o "projects_graphs/$NAME/graph" --include-dev --include-build 2>&1
./target/debug/graph merge -i projects_graphs -o global_graph 2>&1
```
