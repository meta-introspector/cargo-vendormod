# Workload Processing Plan

This document outlines how to use cargo-vendormod tools on different workloads.

## Overview

| Workload | Location | Type | Complexity |
|----------|----------|------|------------|
| Self | `.` | Rust tool (self-hosting) | Low |
| Rust Toolchain | `workload/workspaces/rust_toolchain/` | Rust ecosystem | Medium |
| Cargo-Rail | `workload/workspaces/cargo-rail/` | 50+ crates workspace | High |
| Solana | `test_solana_processing/workspaces/` | On-chain programs | Medium |

## Prerequisites

### 1. Configure mirrors directory

```bash
# Create mirrors directory
mkdir -p ~/git/host

# Or set in config
echo 'mirrors-dir = "~/git/host"' > vendormod.toml
```

### 2. Ensure git is available

```bash
# Verify git path
which git
# Or configure in vendormod.toml:
# git-path = "/usr/bin/git"
```

---

## Workload 1: Self (cargo-vendormod)

The project itself - simplest case as it's already a cargo project.

### Step 1: Build dependency graph

```bash
cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod
cargo run --bin graph -- build \
  --workspace-path . \
  --include-dev \
  --output-dir ./analysis/self
```

### Step 2: Analyze dependencies

```bash
cargo run --bin graph -- analyze \
  --input-path ./analysis/self/graph.json \
  --output-dir ./analysis/self
```

### Step 3: Generate visualization

```bash
cargo run --bin graph -- visualize \
  --input-path ./analysis/self/graph.json \
  --output-path ./analysis/self/graph.dot
```

### Step 4: Process crates (if needed)

```bash
cargo run --bin processing -- crates \
  --workspace-path . \
  --output-dir ./processed/self \
  --generate-flakes
```

### Expected Output

```
analysis/self/
├── graph.json          # Full dependency graph
├── analysis.json       # Metrics and patterns
└── graph.dot          # Graphviz visualization

processed/self/
└── flakes/            # Nix flakes per crate
```

---

## Workload 2: Rust Toolchain

Location: `workload/workspaces/rust_toolchain/`

Contains: cargo, rustc-apfloat, rustc-demangle, compiler-builtins, cargo_metadata

### Step 1: Discover workspace members

```bash
# Check workspace structure
ls workload/workspaces/rust_toolchain/
```

### Step 2: Build graph

```bash
cargo run --bin graph -- build \
  --workspace-path ./workload/workspaces/rust_toolchain \
  --output-dir ./analysis/rust_toolchain
```

### Step 3: Analyze toolchain patterns

```bash
cargo run --bin graph -- analyze \
  --input-path ./analysis/rust_toolchain/graph.json \
  --output-dir ./analysis/rust_toolchain
```

### Step 4: Process in topological order

```bash
cargo run --bin processing -- crates \
  --workspace-path ./workload/workspaces/rust_toolchain \
  --output-dir ./processed/rust_toolchain \
  --layered-processing \
  --generate-flakes
```

### Key Insight

Rust toolchain crates often have:
- Heavy internal dependencies
- Special build requirements (llvm, etc.)
- Often bootstrapped from previous versions

---

## Workload 3: Cargo-Rail Style

Location: `workload/workspaces/cargo-rail/`

Contains: 50+ crates including serde, tokio, reqwest, etc.

### Step 1: Full graph analysis

```bash
cargo run --bin graph -- build \
  --workspace-path ./workload/workspaces/cargo-rail \
  --include-dev \
  --expand-features \
  --output-dir ./analysis/cargo_rail
```

### Step 2: Partition for parallel processing

```bash
cargo run --bin graph -- partition \
  --input-path ./analysis/cargo_rail/graph.json \
  --partition-count 8 \
  --algorithm kaminpar \
  --output-dir ./analysis/cargo_rail/partitions
```

### Step 3: Process with layers

External dependencies first, then workspace members:

```bash
# Process layer 1 (external deps)
cargo run --bin processing -- crates \
  --workspace-path ./workload/workspaces/cargo-rail \
  --output-dir ./processed/cargo_rail \
  --layered-processing \
  --layer 1 \
  --max-parallel 8

# Process layer 2 (workspace members)
cargo run --bin processing -- crates \
  --workspace-path ./workload/workspaces/cargo-rail \
  --output-dir ./processed/cargo_rail \
  --layered-processing \
  --layer 2 \
  --max-parallel 8
```

### Step 4: Run complete workflow

```bash
cargo run --bin processing -- workflow \
  --workspace-path ./workload/workspaces/cargo-rail \
  --output-dir ./output/cargo_rail \
  --workflow-type standard
```

### Generate report

```bash
cargo run --bin processing -- report \
  --workspace-path ./workload/workspaces/cargo-rail \
  --output-dir ./reports/cargo_rail
```

---

## Workload 4: Solana Programs

Location: `test_solana_processing/workspaces/`

Contains: secp256k1-program, loader-v2, address-lookup-table, etc.

### Step 1: Build graph

```bash
cargo run --bin graph -- build \
  --workspace-path ./test_solana_processing/workspaces \
  --output-dir ./analysis/solana
```

### Step 2: Analyze TOML structures

```bash
cargo run --bin graph -- toml-structure \
  --input-path ./analysis/solana/graph.json \
  --output-dir ./analysis/solana/toml
```

### Step 3: Process programs

```bash
cargo run --bin processing -- crates \
  --workspace-path ./test_solana_processing/workspaces \
  --output-dir ./processed/solana \
  --layered-processing
```

### Solana-Specific Notes

- Many programs have similar dependencies (solana-sdk, solana-program)
- Cross-program invocations create complex dependency graphs
- BPF compilation requires special toolchain

---

## End-to-End Pipeline

### Complete processing script

```bash
#!/bin/bash
# process_all_workloads.sh

WORKLOADS=("self" "rust_toolchain" "cargo_rail" "solana")
PATHS=("." "./workload/workspaces/rust_toolchain" "./workload/workspaces/cargo_rail" "./test_solana_processing/workspaces")

for i in "${!WORKLOADS[@]}"; do
  WORKLOAD="${WORKLOADS[$i]}"
  PATH="${PATHS[$i]}"
  
  echo "=== Processing $WORKLOAD ==="
  
  # Build graph
  cargo run --bin graph -- build \
    --workspace-path "$PATH" \
    --output-dir "./analysis/$WORKLOAD"
  
  # Analyze
  cargo run --bin graph -- analyze \
    --input-path "./analysis/$WORKLOAD/graph.json" \
    --output-dir "./analysis/$WORKLOAD"
  
  # Process
  cargo run --bin processing -- crates \
    --workspace-path "$PATH" \
    --output-dir "./processed/$WORKLOAD" \
    --generate-flakes
  
  echo "=== $WORKLOAD complete ==="
done
```

---

## Configuration Template

Create `vendormod.toml` in project root:

```toml
# Paths (use absolute or relative)
vendor-dir = "vendor"
submodules-dir = "submodules"
mirrors-dir = "~/git/host"

# Git settings
git-path = "git"
target-branch = "main"
version-branch-format = "v{}"
create-version-branches = false

# Performance
default-threads = 8
```

---

## Common Commands Reference

| Task | Command |
|------|---------|
| Build graph | `cargo run --bin graph -- build --workspace-path .` |
| Analyze graph | `cargo run --bin graph -- analyze --input-path ./graph.json` |
| Visualize | `cargo run --bin graph -- visualize --input-path ./graph.json -o graph.dot` |
| Partition | `cargo run --bin graph -- partition --input-path ./graph.json -n 8` |
| Process crates | `cargo run --bin processing -- crates --workspace-path . --generate-flakes` |
| Process file list | `cargo run --bin processing -- all --input-file crates.txt` |
| Run workflow | `cargo run --bin processing -- workflow --workspace-path .` |
| Generate patches | `cargo run --bin vendoring -- patch` |
| Sync submodules | `cargo run --bin vendoring -- sync` |

---

## Output Directories

```
.
├── analysis/           # Graph data
│   ├── self/
│   ├── rust_toolchain/
│   ├── cargo_rail/
│   └── solana/
├── processed/          # Processed crates
│   ├── self/
│   ├── rust_toolchain/
│   ├── cargo_rail/
│   └── solana/
└── reports/            # Processing reports
```