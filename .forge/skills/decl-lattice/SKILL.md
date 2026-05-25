---
name: decl-lattice
description: Build a code lattice where each declaration becomes an individual buildable crate with its own Cargo.toml, flake.nix, and dependency edges. Use when the user asks to create a code lattice, connect declarations as crates, generate per-decl flakes, or refactor code into kernel operations. Triggers on: "decl lattice", "code lattice", "decl graph", "kernel operation", "micro crate", "crate per declaration".
---

# Decl Lattice

Build a lattice of code where each declaration becomes its own crate,
connected by dependency edges representing function calls.

## Prerequisites

- `split-decls` skill output: per-project declaration files
- Python3 for running the lattice generation scripts
- `graph` binary for lattice analysis

## Pipeline

```
decl-splitter output → build_decl_graph.py → decl_graph.json
                                               ↓
decl_graph.json → gen_decl_lattice.py → decl_lattice/crates/<decl>/
                                           ├── Cargo.toml
                                           ├── flake.nix
                                           └── src/lib.rs
                                               ↓
decl_lattice/Cargo.toml  (workspace, 508 members)
decl_lattice/flake.nix   (all crates as Nix packages)
```

## Step 1 — Build Decl Dependency Graph

```bash
python3 scripts/build_decl_graph.py --input <decls-dir> --output <graph-dir>
```

This scans all declaration files and finds:
- `use crate::<Name>` references between decls
- Function call references matching declaration names
- External crate imports (`use serde::`, `use petgraph::`, etc.)

Output: `decl_graph.json` (nodes=decls, edges=function calls)

## Step 2 — Generate Lattice Crates

```bash
python3 scripts/gen_decl_lattice.py --graph <graph-dir>/decl_graph.json --output <lattice-dir>
```

For each declaration, generates:

### Cargo.toml
- `path` dependencies to other decls in the lattice
- `*` dependencies for external crates
- Edition 2021, version 0.1.0

### flake.nix
- Inputs: nixpkgs, flake-utils, rust-overlay
- `rustPlatform.buildRustPackage`
- Requires `Cargo.lock` (generate via `cargo generate-lockfile`)

### src/lib.rs
- The original declaration file (self-contained with all imports)

## Step 3 — Build the Lattice

```bash
# Via cargo (all crates)
cd decl_lattice && cargo check

# Via Nix (single crate)
cd decl_lattice && cargo generate-lockfile
nix build --impure .#<decl-name>

# Via Nix (workspace flake)
nix build --impure ./decl_lattice#<decl>
```

## Lattice JSON Format

```json
{
  "nodes": 508,
  "edges": 2970,
  "depth": 41,
  "kernel_ops": {
    "<op-type>": {
      "type": "<operation>",
      "deps": ["<dependent-decls>"]
    }
  }
}
```

Each node = one kernel operation (function, struct, trait, etc.)
Each edge = function call dependency between operations

## Example Lattice Chain

```
build             (depends on: lines, args, action)
  └── lines       (depends on: none — leaf node)
      └── args    (depends on: clap_types)
          └── clap_types  (depends on: none — foundational)
```

## Scaling

To extend the lattice to new projects:

```bash
# 1. Split declarations on the new project
DS=... && find <project> -name '*.rs' | xargs -P4 $DS -i {} -o /tmp/decls_<p>/{}

# 2. Build decl graph
python3 scripts/build_decl_graph.py --input /tmp/decls_<p> --output <graph-dir>

# 3. Generate lattice crates
python3 scripts/gen_decl_lattice.py --graph <graph-dir>/decl_graph.json --output decl_lattice/extra

# 4. Add to workspace Cargo.toml
```

## One Kernel Operation Per Node

Each declaration represents exactly one kernel operation:

| Pattern | Kernel Op | Example Decl Name |
|---------|-----------|-------------------|
| `fn add*` | ADD | `add_node_to_graph` |
| `fn parse*` | PARSE | `parse_crate_name` |
| `fn compute*` | COMPUTE | `compute_bare_repo_path` |
| `fn normalize*` | NORMALIZE | `normalize_url_to_flat_path` |
| `fn extract*` | EXTRACT | `extract_repo_name` |
| `fn sort*` / `fn order*` | SORT | `sort_build_order` |
| `fn merge*` | MERGE | `merge_graphs` |
| `fn build*` | BUILD | `build_dependency_graph` |
| `fn clone*` | CLONE | `clone_bare_repo` |
