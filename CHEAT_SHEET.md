# Cargo-Vendormod Cheat Sheet

## 🚀 Quick Start

```bash
# Build dependency graph
cargo-vendormod global-graph build --workspace-path .

# Process crates
cargo-vendormod process-crates --workspace-path . --generate-flakes

# Full workflow
cargo-vendormod run-workflow --workspace-path .

# Git submodules
cargo-vendormod vendoring
cargo-vendormod sync
```

---

## 📊 Global Dependency Graph

### Build Graph
```bash
cargo-vendormod global-graph build \
  --workspace-path <PATH> \
  --include-dev \
  --include-build \
  --expand-features \
  --output-dir <DIR>
```

### Analyze Graph
```bash
cargo-vendormod global-graph analyze \
  --input-path <FILE> \
  --output-dir <DIR>
```

### Visualize
```bash
cargo-vendormod global-graph visualize \
  --input-path <FILE> \
  --output-path <FILE>

dot -Tpng graph.dot -o graph.png
```

### TOML Structure
```bash
cargo-vendormod global-graph toml-structure \
  --input-path <FILE> \
  --output-dir <DIR>
```

### Partition
```bash
cargo-vendormod global-graph partition \
  --input-path <FILE> \
  --partition-count 8 \
  --algorithm KaMinPar \
  --output-dir <DIR>
```

---

## 🏗️ Processing

### Process Crates (Layered)
```bash
cargo-vendormod process-crates \
  --workspace-path <PATH> \
  --output-dir <DIR> \
  --generate-flakes \
  --compile-standalone \
  --layered-processing
```

### Process All Crates
```bash
cargo-vendormod process-all-crates \
  --input-file <FILE> \
  --output-dir <DIR> \
  --max-parallel 4
```

### Generate Report
```bash
cargo-vendormod generate-report \
  --input-file <FILE> \
  --output-dir <DIR>
```

---

## 🚀 Workflows

### Run Workflow
```bash
cargo-vendormod run-workflow \
  --workspace-path <PATH> \
  --output-dir <DIR> \
  --workflow-type standard
```

**Types:** `standard` | `minimal` | `ci`

### Workspace Analysis
```bash
cargo-vendormod workspace \
  --workspace-path <PATH> \
  --recursive
```

### Workload Processing
```bash
cargo-vendormod workload \
  --workspace-path <PATH> \
  --fork-dir <DIR> \
  --zkperf
```

---

## 🔄 Git Operations

### Vendoring → Submodules
```bash
cargo-vendormod vendoring
```

### Fetch Upstream
```bash
cargo-vendormod fetch-upstream
```

### Rebase
```bash
cargo-vendormod rebase
```

### Releases
```bash
cargo-vendormod releases
```

### Patch
```bash
cargo-vendormod patch
```

### Status
```bash
cargo-vendormod status
```

### Full Sync
```bash
cargo-vendormod sync
```

---

## ✏️ Edit

### Edit Workspace
```bash
cargo-vendormod edit \
  --sort \
  --add-missing \
  --remove-unused \
  --update-versions
```

---

## 🔧 Fix

### Fix Cargo.toml
```bash
cargo-vendormod fix-cargo-toml
```

### Fix Edition
```bash
cargo-vendormod fix-edition \
  --base-dir <PATH> \
  --edition 2021
```

---

## ☁️ Nix Builds

### Build Pipeline
```bash
cargo-vendormod nix-build-pipeline \
  --flake-dir <DIR> \
  --max-parallel 8 \
  --max-retries 2 \
  --timeout-seconds 3600 \
  --output-dir <DIR> \
  --log-dir <DIR>
```

### Error Report
```bash
cargo-vendormod generate-error-report \
  --results-file <FILE> \
  --output-path <FILE>
```

---

## 🌍 Global Options

```bash
--root-dir PATH              # Root directory
--manifest-path PATH         # Cargo.toml path
--submodules-path PATH       # Submodules directory
--mirrors-path PATH          # Git mirrors
--vendor-dir PATH            # Vendor directory
--target-branch BRANCH       # Target branch
--create-version-branches    # Create version branches
--version-branch-format FMT  # Branch format (v{})
--dry-run                    # Dry run
--output-file FILE           # JSON output file
--verbose                    # Verbose output
--help                       # Show help
```

---

## 📈 Common Workflows

### New Workspace
```bash
cargo-vendormod global-graph build --workspace-path . --include-dev
cargo-vendormod global-graph analyze --input-path graph.json
cargo-vendormod global-graph visualize --input-path graph.json
```

### Process with Nix
```bash
cargo-vendormod process-crates \
    --workspace-path . \
    --output-dir ./processed \
    --generate-flakes \
    --compile-standalone
```

### Git Submodules
```bash
cargo-vendormod vendoring
cargo-vendormod sync
```

### Full Workflow
```bash
cargo-vendormod run-workflow --workspace-path . --workflow-type standard
```

### Parallel Builds
```bash
cargo-vendormod process-crates --workspace-path . --generate-flakes
cargo-vendormod nix-build-pipeline --flake-dir ./processed --max-parallel 8
```

### Edit Dependencies
```bash
cargo-vendormod edit --sort --add-missing --remove-unused --update-versions
```

---

## 📁 Output Files

| Command | Output |
|---------|--------|
| `global-graph build` | `graph.json` |
| `global-graph analyze` | `analysis_report.md` |
| `global-graph visualize` | `graph.dot`, `.png`, `.svg` |
| `global-graph toml-structure` | `toml_structure_analysis.md`, `.dot` |
| `global-graph partition` | `partitioned_graph.json`, partition files |
| `process-crates` | `flake.nix`, binaries |
| `run-workflow` | All outputs |

---

## ⚡ Performance

```bash
# Reduce memory
cargo-vendormod process-crates --workspace-path . --max-parallel 2

# Skip compilation
cargo-vendormod run-workflow --workspace-path . --workflow-type minimal

# Dry run
cargo-vendormod --dry-run global-graph build --workspace-path .
```

---

## 🐛 Troubleshooting

```bash
# Permission denied
chmod -R u+rwx ./processed ./analysis

# Out of memory
cargo-vendormod --max-parallel 2 process-crates --workspace-path .

# Not in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Verbose output
cargo-vendormod --verbose <command>
```

---

## 🎯 Quick Reference

```bash
# Help
cargo-vendormod --help
cargo-vendormod global-graph --help

# Version
cargo-vendormod --version

# Dry run
cargo-vendormod --dry-run <command>

# Verbose
cargo-vendormod --verbose <command>
```

---

## 📚 More Info

- **README.md** - Overview and installation
- **GETTING_STARTED.md** - Tutorial
- **USER_GUIDE.md** - Detailed guide
- **CLI_REFERENCE.md** - Complete reference
- **PROJECT_PROGRESS_TRACKER.md** - Status

---

## 💡 Tips

1. Always start with `global-graph build`
2. Use `--dry-run` to preview
3. Use `--verbose` for debugging
4. Save `graph.json` for history
5. Try different partition algorithms
6. Use `--layer` to process selectively
7. Automate with CI
8. Document your workflow

---

🎉 **Ready to go!** 🚀
