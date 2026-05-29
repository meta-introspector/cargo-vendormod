# cargo-vendormod — Session Handoff Summary

**Date:** 2026-05-25
**Repo:** `/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod`
**Master Plan:** `plans/2026-05-25-cargo-vendormod-master-plan-1.0.md`
**Skills:** `.forge/skills/` (6 skills)

---

## 1. What Was Built

### Phase 1: Build
- cargo-vendormod compiled into **35 binaries, 1 library** (Rust 1.94.1)
- Pure Rust — no shell scripts, no `find`, no Python in the core toolchain

### Phase 2: Per-Project Graphs
- `graph build` scans any Cargo workspace using `cargo_metadata::resolve.nodes`
- Fallback: pure-Rust Cargo.lock parser (works without `cargo` binary, no network)
- **38 projects analyzed**, from 4 source locations:
  - `~/projects/` — 27 hand-curated Rust projects
  - `~/nix/.../ragit/` — 457 nodes, 979 edges
  - `~/nix/.../streamofrandom/` — 11 subprojects, ~1,093 nodes total
  - `~/experiments/.../arti-tor-rs/` — 842 nodes, 3,269 edges

### Phase 3: Global Graph Merge
- `graph merge` aggregates all per-project graphs into one
- Produces: `global_graph.json`, `build_order.json`, `upgrade_plan.json`, `mirrors.json`, `projects.json`

| Metric | v1 | v2 | v3 |
|--------|:--:|:--:|:--:|
| Unique crates | 2,518 | 4,211 | **4,653** |
| Dependency edges | 20,480 | 21,250 | **22,304** |
| Projects | 27 | 27 | **38** |
| Upgrade candidates | 674 | 688 | **715** |

### Phase 4: Nix Store Linking (crate2nix)
- Each dependency crate = **separate Nix store path**
- `rustc --extern` points to `/nix/store/...` `.rlib` files — no `~/.cargo/registry/` (4.7GB avoided)
- **22 project Cargo-generated.nix** files (385,704 lines of Nix)
- **2,124 minimal flake.nix** files (one per unique crate)
- **233 bare git repos** (4 from global graph + 229 from scanner mirror)

### Phase 5: Declaration Splitting
- `decl-splitter` extracts individual declarations from `.rs` files
- **947,362 declaration files** split across all processed projects

### Phase 6: Decl Lattice
- **508 declarations** from cargo-vendormod converted to individual buildable crates
- Each with: `Cargo.toml` (path deps + extern deps), `flake.nix`, `src/lib.rs`
- **2,970 dependency edges** between lattice nodes
- Location: `decl_lattice/crates/<name>/`

---

## 2. Key Commits (10 most recent)

| SHA | Description |
|-----|-------------|
| `bebfced` | Master plan + 6 reusable skills |
| `50ed815` | Global graph v3: ragit, arti-tor-rs, streamofrandom (4,653 crates) |
| `418e94a` | Decl lattice: 508 declarations as individual crates + flakes |
| `0c3f78f` | Nix-store crate linking verified (each dep = separate store path) |
| `9159f72` | crate2nix-zos applied to all 27 projects |
| `bb26243` | Global graph v2 + fractran-vm + boa + scanner + 947K decls |
| `eb7a6c1` | Global graph merge with bare repos, cargo config patch, crate flakes |
| `b157d2f` | Pure-Rust Cargo.lock parser + Nix per-project graph derivations |
| `1feaa89` | Flake: 35 binaries as packages+apps |
| `101b372` | Fix: use cargo-metadata resolve.nodes for correct deps |

---

## 3. Skills (`.forge/skills/`)

| Skill | File | What It Does |
|-------|------|-------------|
| **build-graph** | `SKILL.md` + `scripts/scan_one.sh` | `graph build` on any project |
| **merge-graphs** | `SKILL.md` | Merge per-project graphs → global |
| **crate2nix-build** | `SKILL.md` | Build via crate2nix in Nix sandbox |
| **split-decls** | `SKILL.md` | Run `decl-splitter` on `.rs` files |
| **decl-lattice** | `SKILL.md` + `scripts/build_lattice.sh` | Generate micro-crate lattice from decls |
| **scan-index** | `SKILL.md` + `scripts/batch_scan.sh` | Batch-scan 74K workspace roots from `cargos.txt` |

---

## 4. Key Files & Outputs

| Path | Size | Contents |
|------|------|----------|
| `projects_graphs/` | 29 MB | Per-project dependency graphs (38 projects) |
| `global_graph/` | 29 MB | Global graph v3: 4,653 nodes, 22,304 edges |
| `global_graph/flakes/` | — | 2,124 minimal `flake.nix` per unique crate |
| `global_graph/bare_repos/` | — | 4 git source mirrors |
| `global_graph/cargo_mirror_config.toml` | — | Patch to redirect git URLs to local mirrors |
| `decl_lattice/` | 7.2 MB | 508 micro-crates (each = one declaration) |
| `decl_graph/` | 260 KB | Dependency graph of the 508 decls |
| `crate2nix_output/` | 14 MB | 22 project-level `Cargo-generated.nix` |
| `.forge/skills/` | 144 KB | 6 reusable skill workflows |

---

## 5. How to Use

```bash
# Build locally
cargo build --bin graph

# Analyze a single project
./target/debug/graph build -w <project-dir> -o <output-dir>

# Merge all project graphs
./target/debug/graph merge -i projects_graphs -o global_graph

# Build via Nix (crate2nix)
nix build --impure ./crate2nix_output#nginx-generator

# Split declarations
decl-splitter -i src/main.rs -o target/decls/

# Build decl lattice
cd decl_lattice && cargo check

# Batch scan from index
bash .forge/skills/scan-index/scripts/batch_scan.sh \
  /home/mdupont/2026/05/25/cargos.txt \
  projects_graphs 0 50
```

---

## 6. Current Gaps

| Gap | Details |
|-----|---------|
| **Coverage** | 38 / 74,009 workspace dirs on disk (0.05%) |
| **Decl lattice** | Only covers cargo-vendormod's 508 decls — not applied to ragit (3,735), arti (7,335), etc. |
| **Nix builds** | Only 4/22 projects proven to build via crate2nix |
| **Bare repo sync** | 4 git sources mirrored but not populated with content |
| **Platform support** | `x86_64-linux` only in flake |

---

## 7. Next Steps (from Master Plan Phase 8)

1. **Scaffold full project structure** — run `scan-index` in batches of 50 across the 74K workspace roots
2. **Integrate into CI** — wrap `graph merge` + `nix build` in a GitHub action
3. **Expand decl lattice** — run `split-decls` + `decl-lattice` on ragit, arti-tor-rs, streamofrandom
4. **Automate upgrade plan** — run `cargo update` on each project following `upgrade_plan.json`
5. **Populate bare repos** — fetch actual git objects into the 233 bare mirror repos
6. **Cross-platform** — add `aarch64-linux` support to flake
