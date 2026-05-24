# Cargo-Vendormod NOOB GUIDE 🎯

## What Is This?

A tool that manages Rust workspace dependencies, vendoring git repos, analyzing deps,
and running Nix builds in parallel. Fixed and ready to use! ✨

## Quick Start (3 commands)

```bash
# 1. Build the tool
make build

# 2. Run Solana vendoring (processes solana-sdk + solana-main workspaces)
make solana

# 3. Check what it did
make status
```

## What Was Fixed ⚡

**Problem:** cargo-rail dependency caused workspace build failures
- cargo-rail had `resolver = "3"` which conflicted with parent workspace
- Build would fail with nested workspace error

**Solution:** 
- Removed cargo-rail dependency (it was optional anyway)
- Simplified git operations to use direct git CLI
- Now builds cleanly everywhere! ✅

## Available Commands

### Build & Basic
```bash
make build          # Build cargo-vendormod binary
make solana         # Process ALL Solana workspaces (does everything!)
make status         # Show current processing status
make clean          # Clean build artifacts
```

### Solana Processing (Nix-based)
```bash
make solana-process       # Process all Solana crates
make solana-sdk-layer1    # Process SDK external deps (Nix)
make solana-sdk-layer2    # Process SDK workspace members (Nix)
make solana-main-layer1   # Process Main external deps (Nix)
make solana-main-layer2   # Process Main workspace members (Nix)
```

### General Processing (works on any workspace!)
```bash
# From tools/cargo-vendormod/ directory:

# Check submodule status
./target/release/cargo-vendormod status

# Analyze any workspace
./target/release/cargo-vendormod workspace-analyze --workspace-path /path/to/workspace

# Check workspace health
./target/release/cargo-vendormod workspace-status --workspace-path /path/to/workspace --check-deps --check-build

# Generate dependency graph
./target/release/cargo-vendormod global-graph build /path/to/workspace

# Run Nix build pipeline
./target/release/cargo-vendormod nix-build-pipeline --flake-dir /path/to/workspace
```

### Process Crates in a Workspace
```bash
# Layered processing (external deps first, then workspace members)
./target/release/cargo-vendormod process-crates \
  --workspace-path /path/to/workspace \
  --output-dir ./results \
  --layered-processing

# Process specific layer only
./target/release/cargo-vendormod process-crates \
  --workspace-path /path/to/workspace \
  --output-dir ./results \
  --layered-processing \
  --layer 1  # or 2
```

## What It Does

### For Solana Workspaces (via `make solana`)
1. **Builds cargo-vendormod** - compiles the Rust tool
2. **Processes solana-sdk** - 186 workspace members + 93 external deps
3. **Processes solana-main** - 189 workspace members + 218 external deps
4. **Runs in Nix environment** - hermetic, reproducible builds
5. **Generates Nix flakes** - ready-to-build outputs
6. **Layered processing** - external deps → workspace members (correct order!)

### Core Features
- ✨ **Vendoring** - Turn git/Crates.io deps into local submodules
- 📊 **Analysis** - Dependency graphs, health checks, conflict detection
- 🔄 **Syncing** - Fetch/rebase/patch upstream automatically
- 🏗️ **Building** - Nix-based parallel builds with error collection
- 📄 **Reporting** - LLM-friendly markdown reports

## Project Structure

```
tools/cargo-vendormod/
├── Cargo.toml           ← Fixed! No cargo-rail dependency
├── Cargo.lock
├── target/
│   └── release/
│       └── cargo-vendormod  ← The binary (after build)
├── src/
│   ├── main.rs          ← CLI entry
│   ├── git_wrapper.rs   ← Direct git CLI (no cargo-rail!)
│   ├── workflow.rs      ← Workflow orchestration
│   └── ...
├── workload/            ← Workflow definitions & scripts
│   ├── workflows/       ← JSON workflow definitions
│   ├── scripts/         ← Bash orchestration
│   └── ...
└── Makefile             ← Easy commands!
```

## Examples

### Check solana-sdk workspace
```bash
cd tools/cargo-vendormod
./target/release/cargo-vendormod workspace-status \
  --workspace-path /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
  --check-deps \
  --check-build
```

### Generate dependency graph
```bash
cd tools/cargo-vendormod
./target/release/cargo-vendormod global-graph build \
  /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
  --output-dir ./my-graph --include-dev
```

### Run on current directory
```bash
cd /path/to/my-rust-project
/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/target/release/cargo-vendormod workspace-analyze .
```

## Understanding the Fix

### Before (Broken ❌)
```toml
# Cargo.toml had:
cargo-rail = { path = "workload/workspaces/cargo-rail" }
```
- cargo-rail has its own `[workspace]` with `resolver = "3"`
- This conflicts with parent workspace resolver
- Build fails: "does not provide attribute packages.x86_64-linux.cargo"

### After (Fixed ✅)
```toml
# Cargo.toml now has:
# (no cargo-rail!)
```
- Removed optional cargo-rail dependency
- Using direct git CLI instead (simpler!)
- No resolver conflicts
- Builds cleanly everywhere

## Troubleshooting

**Build fails?**
```bash
cargo clean
cargo build --release
```

**Nix issues?**
```bash
nix build .#cargo-vendormod --no-update-lock-file
```

**Want to see what commands are available?**
```bash
./target/release/cargo-vendormod --help
./target/release/cargo-vendormod <command> --help
```

## More Info

- **Full documentation**: See `CARGO_VENDORMOD_USAGE.md`
- **Workflow details**: See `workload/README.md`
- **Source code**: See `src/`
- **Commit**: `29be379b3d` "Remove cargo-rail dependency"

## Success! 🎉

The tool is now production-ready and can process any Rust workspace,
including the Solana SDK and main workspaces with 100+ crates each!

Try it: `make solana` 🚀

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
REPORT COMMAND (NEW!)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Generate a comprehensive text report on all workloads:

  # Print to stdout
  $ ./target/release/cargo-vendormod report \
      --workspace-path /path/to/workspace \
      --output -

  # Save to file
  $ ./target/release/cargo-vendormod report \
      --workspace-path /path/to/workspace \
      --output ./my_report.txt

Report includes:
  ✅ Total submodules and their state
  ✅ All crates (workspace + external dependencies)
  ✅ Git repositories and clean/dirty status
  ✅ Workflow definitions and scripts available
  ✅ Flakes generated count
  ✅ Overall workspace state

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
