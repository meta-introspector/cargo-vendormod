# cargo-vendormod Master Plan

## Overview

Transform cargo-vendormod into a complete dependency analysis and code lattice toolchain.
The system builds dependency graphs from Cargo projects, merges them into a global graph,
splits declarations into individual lattice nodes, and generates Nix build infrastructure
for all crates — all in pure Rust with Nix-store-verified linking.

## Current State (2026-05-25)

| Capability | Status | Details |
|-----------|--------|---------|
| `graph build` | ✅ | Pure-Rust Cargo.lock parser, 2 fallback modes |
| `graph merge` | ✅ | Merges graphs, topo sort, upgrade plan, bare repos, flakes |
| `graph analyze` | ✅ | Full metrics, SCC analysis, density |
| `graph visualize` | ✅ | DOT + SVG output via graphviz |
| `graph partition` | ✅ | K-way graph partitioning |
| `graph toml-structure` | ✅ | Structure analysis from cargo metadata |
| `scanner` binary | ✅ | 8-level recursive git discovery, submodule recursion |
| `decl-splitter` | ✅ | 947K declarations from 27+ projects |
| `crate2nix` derivations | ✅ | 22 projects, 385K lines Nix, 97.5% crate coverage |
| Decl lattice | ✅ | 508 decls as individual crates with flakes |
| Per-project Nix flakes | ✅ | 2,124 crate flakes, 27 project graph derivations |
| Global graph v3 | ✅ | 4,653 crates, 22,304 edges, 38 projects |

## Current Limitations

| Limitation | Fix | Priority |
|-----------|-----|----------|
| Git clone uses 2 code paths (git2 + raw CLI) | Unify into single `GitCLI` struct | High |
| GitWrapper is dead code (3/4 callers bypass it) | Replace with `GitCLI`, remove GitWrapper | High |
| eBPF + strace PID data not cross-referenced | Add `GitSubprocess` marker struct | Medium |
| SELinux analyzer only targets .service files | Extend for git binary policies | Medium |
| repo_sync_lib git2 path unmaintained | Deprecate, point to `GitCLI` | Low |
| Only 38/74K workspace roots scanned | Batch scan pipeline using cargos.txt index | Medium |

## Phase 1 — Core Unification

### Task 1.1: GitCLI struct
- Create `src/git_cli.rs` with `GitCLI { cargo_bin, timeout }` struct
- Implement: `clone(url, path, bare)`, `checkout(ref, path)`, `fetch(remote, path)`, `submodule_update(path)`, `rev_parse(ref, path)`
- All via `std::process::Command` with proper error handling and logging
- Accept: `git2` removal in follow-up phase

### Task 1.2: Route all callers through GitCLI
- `workload.rs:162` — raw `git clone` → `GitCLI::clone()`
- `layer_processor.rs:177` — raw `git checkout` → `GitCLI::checkout()`
- `submodule_discovery.rs:88` — raw `git submodule` → `GitCLI::submodule_update()`
- `workload.rs:198` — raw `git checkout` → `GitCLI::checkout()`
- 48 decls in lattice doing `Command::new("git")` — update templates

### Task 1.3: Remove GitWrapper
- Delete `src/git_wrapper.rs`
- Remove `mod git_wrapper` from `src/lib.rs`
- Update any remaining callers

**Estimated impact**: -200 lines dead code, +180 lines new GitCLI

## Phase 2 — Monitoring Cross-Reference

### Task 2.1: GitSubprocess struct
- Add to `src/multi_tool_correlator.rs`: `GitSubprocess { pid, url, args, start_time, duration, syscall_count, io_bytes }`
- Populate from `MetadataCommand` output (cargo_metadata provides git source URLs)

### Task 2.2: PID cross-reference
- eBPF tracks `clone`/`execve` syscalls per PID
- strace tracks `open`/`read`/`write` per PID
- Join on PID → "git clone of X took Y ms, Z syscalls"

### Task 2.3: SELinux policy extension
- `src/selinux_analyzer.rs`: add pass generating `allow git_t *:process fork;`
- Generate `git_domain` policy for each unique git invocation pattern

**Estimated impact**: +150 lines correlator, +60 lines SELinux

## Phase 3 — Scale Analysis

### Task 3.1: Batch project scanning
- Use `scanner --scan-dir <dir> --mirrors-dir <mirror> --max-iterations 8`
- For each discovered Cargo workspace root: `graph build -w <root> -o <out>`

### Task 3.2: Index-driven scanning
- Process `/home/mdupont/2026/05/25/cargos.txt` (121K Cargo.tomls)
- Filter: unique workspace roots with `[workspace]` + `Cargo.lock`
- Batch via `make scan-index N=<name>` or automated pipeline
- Target: 500 → 5,000 → 50,000 workspace roots

### Task 3.3: Decl lattice at scale
- Run `decl-splitter` on all new `.rs` sources from scanned projects
- Run `scripts/build_decl_graph.py` on each project's decls
- Run `scripts/gen_decl_lattice.py` to produce crate-per-decl
- Goal: 500 → 100K → 1M lattice nodes

**Estimated impact**: 38 → 500+ projects in global graph

## Phase 4 — Nix Build Infrastructure

### Task 4.1: Crate2nix universal flake
- `crate2nix_output/flake.nix` already wraps 22 projects
- Add `flake.lock` pinning to all inputs
- Verify build for each project: `nix build --impure .#<project>`
- Target: all 38+ projects buildable

### Task 4.2: Lattice crate builds
- 508 decl lattice crates need Cargo.lock generated
- `cargo generate-lockfile` in each crate dir
- Nix build: `nix build --impure ./decl_lattice#<decl>`
- Verify at least leaf decls build (no circular deps)

### Task 4.3: Global Nix CI
- `nix flake check` passes
- `nix build .#global-graph` produces merged graph in Nix store
- `nix run .#graph -- build -w <path> -o <out>` works for any path

## Phase 5 — Intelligent Refactoring

### Task 5.1: Upgrade plan execution
- `global_graph/upgrade_plan.json` lists 715 version mismatches
- For each candidate: update Cargo.toml, regenerate graph, verify build
- Priority: semver-incompatible patches first

### Task 5.2: Tree-shaking via traces
- `workload_optimizer.rs` already uses strace/ebpf to prune
- Run full trace on a build → mark unused crate paths
- Prune from `Cargo-generated.nix` and global graph

### Task 5.3: Code lattice service decomposition
- Each lattice node becomes an independent service
- Interface: CBOR over Unix socket (inspired by zos-plugins)
- Build order from topological sort ensures correct startup
- One kernel operation per service (`+`, `*`, `log`, `mod`, etc.)

## Verification

Each task must verify:
1. **Compiles**: `cargo build --bin graph` no errors
2. **Runs**: `graph build -w . -o /tmp/verify` completes < 30s
3. **Pure Rust**: no shell scripts, no find, no Python in production path
4. **Nix-store compatible**: `nix build --impure .#<package>` works in sandbox

## Schedule

| Phase | Tasks | Expected duration |
|-------|-------|-------------------|
| Phase 1 — Git unify | 1.1, 1.2, 1.3 | 2-3 sessions |
| Phase 2 — Monitoring | 2.1, 2.2, 2.3 | 1-2 sessions |
| Phase 3 — Scale | 3.1, 3.2, 3.3 | 3-5 sessions |
| Phase 4 — Nix infra | 4.1, 4.2, 4.3 | 1-2 sessions |
| Phase 5 — Refactor | 5.1, 5.2, 5.3 | 3-4 sessions |

Total: ~10-16 sessions to full production readiness.
