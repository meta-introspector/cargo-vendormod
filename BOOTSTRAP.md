# Bootstrap Vendoring

This document describes how to use cargo-vendormod to bootstrap a new root repository with all submodules and dependencies.

## Overview

The bootstrap feature discovers and vendors:
- All git submodules (recursively up to 8 levels deep)
- All git dependencies from Cargo.lock
- Dev, build, and optional dependencies

All repositories are flattened to `host/owner/repo` structure in the target directory.

## Usage

### Basic Command

```bash
cargo run --release -- \
  --manifest-path ./Cargo.toml \
  --submodules-path newroot/vendor/submodules \
  --mirrors-path newroot/vendor/mirrors \
  --root-dir newroot \
  --source-repo . \
  --include-dev \
  --include-build \
  --include-optional \
  vendoring
```

### Or use the Makefile target

```bash
make bootstrap
```

## CLI Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--root-dir` | Target root directory for vendoring | `.` |
| `--submodules-path` | Where to store cloned submodules | `submodules` |
| `--mirrors-path` | Where to store bare git mirrors | `/home/mdupont/git/host` |
| `--source-repo` | Source repository to discover submodules from | `.` |
| `--include-dev` | Include dev dependencies | `true` |
| `--include-build` | Include build dependencies | `true` |
| `--include-optional` | Include optional dependencies | `true` |
| `--manifest-path` | Path to Cargo.toml | auto-detect |
| `--target-branch` | Branch to use for submodules | `main` |
| `--dry-run` | Preview without making changes | `false` |

## Directory Structure

After bootstrapping, the target directory will contain:

```
newroot/
├── vendor/
│   └── submodules/
│       ├── github.com/
│       │   ├── meta-introspector/
│       │   │   ├── cargo-edit/
│       │   │   ├── krates/
│       │   │   └── zkperf/
│       │   └── loadingalias/
│       │       └── cargo-rail/
│       └── ... (all repos flattened to host/owner/repo)
```

## How It Works

1. **Discover Submodules**: Parse `.gitmodules` from source repository
2. **Recursive Discovery**: Walk into each submodule and find its `.gitmodules` (up to 8 levels deep)
3. **Clone Submodules**: Clone all discovered submodules to target using flat `host/owner/repo` paths
4. **Discover Dependencies**: Parse Cargo.lock from source and all submodules
5. **Filter Dependencies**: Include/exclude based on dev/build/optional flags
6. **Create Actions**: Build vendoring action plan for each discovered git dependency
7. **Execute Vendoring**: Add each repository as a submodule in the target

## Example

To vendor cargo-vendormod itself:

```bash
# From the cargo-vendormod directory
make bootstrap
```

This will:
1. Find submodules: `cargo-edit`, `krates`, `zkperf`, `cargo-rail`
2. Find nested submodules (e.g., inside `zkperf`)
3. Clone all to `newroot/vendor/submodules/`
4. Discover git dependencies from Cargo.lock
5. Add all as submodules in `newroot/`

## Notes

- Mirrors are not created during bootstrap - they will be added in a future step
- Existing submodules in the target directory are skipped (not overwritten)
- The `--source-repo` defaults to current directory (`.`)
- All discovered repositories use the canonical upstream URL for path naming