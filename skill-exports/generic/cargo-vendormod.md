# cargo-vendormod

**Description**: A comprehensive tool for managing complex Rust workspaces, especially monorepos. It helps analyze dependency graphs, process crates in topological order, manage git submodules, and generate Nix flakes.

## Source

- ./skills/cargo-vendormod/SKILL.md

---

# Cargo-Vendormod

## Purpose

`cargo-vendormod` is a suite of tools for managing complex Rust workspaces. It has binaries for dependency graph analysis (`graph`), git submodule management (`vendoring`), and crate processing (`processing`).

## CLI / Script Entrypoints

```bash
# Get help for the main command
cargo-vendormod --help

# Get help for a subcommand
cargo-vendormod <subcommand> --help

# Run a workflow
cargo-vendormod run-workflow --workspace-path .

# Git submodule vendoring operations
cargo-vendormod vendoring

# Sync submodules
cargo-vendormod sync
```

## Rust Library Entrypoints

```rust
use cargo_vendormod::config::Config;
use cargo_vendormod::args::Args;
use cargo_vendormod::global_dep_graph::GlobalDependencyGraphBuilder;
use cargo_vendormod::layer_processor::LayerProcessor;
```

## Key Outcomes

- Dependency graph analysis
- Layered crate processing
- Git submodule management
- Nix flake generation

## Workflow

1.  **Analyze a new workspace:**
    -   `cargo-vendormod global-graph` to build the dependency graph.
    -   `cargo-vendormod visualize-graph` to visualize the graph.
2.  **Process a workspace with Nix:**
    -   `cargo-vendormod process-crates --generate-flakes` to generate Nix flakes for each crate.
3.  **Manage git submodules:**
    -   `cargo-vendormod vendoring init` to initialize submodules.
    -   `cargo-vendormod vendoring sync` to sync submodules.

