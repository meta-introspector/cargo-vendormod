# vendormod-module-tile — Universal Task

*(meta-prompt injected per bafk32aa331a49135aecb51d61d66349442a)*

## 1. Task Context & Reflection

### Prior Work (Reflection)
A previous pi agent completed refactoring of cargo-vendormod's monolithic
`src/main.rs`. This work:
- Moved 26 public items (functions, structs, constants) from `main.rs` → `utils.rs`
- Reduced `main.rs` from 1703 → 646 lines (62% reduction)
- Created `utils.rs` at 1093 lines with 6 organized sections
- Fixed 2 pre-existing undefined functions (`get_defined_workloads`, `handle_scan_delta`)
- All builds pass: `cargo check` — zero errors (lib + 20+ binaries)

### Dependencies (Upstream Modules / Artifacts)
- **Code tree:** `cargo-vendormod/worktrees/refactor-main/src/` (main.rs, utils.rs, lib.rs, 30+ modules)
- **Documentation:** `~/DOCS/VENDORMOD_REFACTOR_MAIN_RS.md` — full refactoring doc
- **Master plan:** `~/dasl/plan.org` — search "update jun 7 — cargo-vendormod main.rs refactored"
- **Tile pattern:** `~/dasl/ipld-car-ipc-shmem-linux/tasks/diagonalize/tile-server/static/diagonalize-tile.html`

### Task Boundary
- **IN SCOPE:** Build a single standalone HTML tile that visualizes the refactored module structure
- **OUT OF SCOPE:** Server-side infrastructure, nginx config, systemd services, database backends

### Project Fit
This tile makes the refactoring visible and navigable. It connects to the DASL architecture by
badging items that relate to: Hecke spectral scoring (hypermorphisms), IPLD shmem storage,
CID computation (content addressing), and dependency graph management. It serves as both
developer documentation and a demonstration of the "tile" UI pattern.

## 2. Required Deliverables

### Core Implementation
- **`vendormod-module-tile.html`** — standalone, zero-dependency HTML file containing:
  - All module data embedded as JSON (no network requests)
  - Dark theme matching DASL color scheme
  - Responsive layout (desktop + mobile)

### Support Modules
The HTML file must implement these four views:

**🌳 Module Tree View**
- Collapsible tree of all 30+ cargo-vendormod modules
- Color-coded by category: core, cli, analysis, atlas, integration
- Import edges: which module imports from which
- `utils.rs` highlighted as the **new** shared dependency hub

**🔄 Refactor Replay View**
- Side-by-side: main.rs before (1703 lines monolithic) vs after (646 lines dispatch)
- 26 items animated flying from main.rs to their 6 destination sections in utils.rs
- Click "replay" to watch the transition

**📊 File Metrics View**
- Bar chart: main.rs 1703→646 reduction
- Pie chart: percentage moved to utils vs stayed
- Per-module line count table

**🔍 Search**
- Filter modules and items by name
- Jump to item in tree on match

### Documentation
- Inline comments in the HTML explaining the data model and rendering approach
- Module header comment with purpose, author, date

### Integration Hooks
- Data `id` attributes on every DOM element for external linking
- URL hash routing: `#module=utils` opens to that module in tree view
- CSS class names following DASL conventions (`.dasl-dark`, `.dasl-accent-cyan`, etc.)

### Validation
- Tile opens correctly in Firefox and Chromium (test with `python3 -m http.server`)
- All 26 refactored items present in the data model
- Search returns correct results for at least: "compute_cid", "FileSample", "handle_scan_index"
- No JavaScript errors in browser console

## 3. Completion Criteria (Universal Definition)

The task is done when ALL of the following are true:

- [ ] **Environment reproducibility** — `flake.nix` exists (already created), `nix develop` enters shell
- [ ] **Skill availability** — skills `rust-async-patterns,nix-flakes,nix,gitnexus-refactoring` loadable
- [ ] **Clean artifact** — `vendormod-module-tile.html` is a single file, zero dependencies
- [ ] **No TODOs** — all views functional, no placeholder text
- [ ] **Version control** — tile committed to task directory
- [ ] **Agent restartability** — another agent can `cd tasks/vendormod-module-tile && ./runme.sh` and continue

## 4. Handoff Requirements

At completion, the agent must produce this handoff block:

### Summary of What Was Done
*(filled by executing agent)*

### Remaining Open Questions
*(filled by executing agent — anything unresolved)*

### Next Recommended Tasks
- [ ] Deploy tile to nginx at `solana.solfunmeme.com/tile/vendormod-modules/`
- [ ] Embed tile in pastebin plugin system (register as `VendormodModulePlugin`)
- [ ] Auto-generate module data from `cargo modules` or `rust-analyzer` output
- [ ] Add live line-count data by parsing current source tree

### Launch Instructions
```bash
cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/worktrees/refactor-main/tasks/vendormod-module-tile
nix develop
python3 -m http.server 8765
# Open http://localhost:8765/vendormod-module-tile.html
```

## 5. Execution Protocol

The agent shall:

1. **Analyze** — restate the task: "Build a standalone HTML tile that visualizes cargo-vendormod's
   refactored module structure, showing the 26-item migration from main.rs to utils.rs
   as an interactive tree, refactor replay, metrics chart, and search."

2. **Plan** — before writing any code, produce a step-by-step plan:
   - Embed module data as JSON
   - Build tree view (recursive rendering)
   - Build refactor replay (CSS animations)
   - Build metrics charts (inline SVG or canvas)
   - Add search/filter
   - Style with DASL dark theme

3. **Execute** — follow the plan, produce `vendormod-module-tile.html`

4. **Revise** — self-check: do all 4 views work? Are all 26 items present?

5. **Deliver** — output the final HTML file and fill in the handoff block above

## 6. Universal Instruction

> **Perform the task, generate all required artifacts, satisfy the completion criteria,
> and prepare a clean handoff so the next agent can continue without loss of context.**

---

## Appendix: Module Data Reference

### Current Project Structure (post-refactoring, src/lib.rs modules)

| Module | Purpose | ~Lines |
|--------|---------|--------|
| `utils` | **NEW** — shared utilities (CID, shmem, sampling, memecache, scan) | 1093 |
| `args` | CLI argument parsing (Commands enum, all Args structs) | ~700 |
| `config` | Configuration management (vendormod.toml) | ~200 |
| `vendoring` | Core vendoring logic (submodule ops) | ~400 |
| `vendoring_cmds` | High-level vendoring command implementations | ~500 |
| `global_dep_graph` | Dependency graph analysis | ~300 |
| `layer_processor` | Topological crate processing | ~300 |
| `workflow` | High-level workflow orchestration | ~200 |
| `git_wrapper` | Git operations | ~200 |
| `lockfile_parser` | Cargo.lock parsing | ~150 |
| `group_atlas` | Finite simple groups atlas | ~200 |
| `visualization` | Atlas tile rendering | ~200 |
| `pastbin_atlas` | Pastbin integration | ~150 |
| `workload_processor` | Workload performance analysis | ~200 |
| `warm_manager` | Directory warming commands | ~150 |
| `nora_indexer` | NORA index building | ~200 |
| `nur_flake` | NUR flake.nix generation | ~150 |
| `crate_flake` | Per-crate flake generation | ~100 |
| `monolith_split` | Lean4 monolith splitter | ~100 |
| `repo_sync_lib` | Repository sync | ~100 |
| `submodule_discovery` | Git submodule discovery | ~150 |

### The 26 Items Moved into `src/utils.rs` (6 sections)

**1. CID & Hash**
- `compute_cid()` — SHA-256 CID-like hash

**2. IPLD Shmem** 🧬
- `SHMEM_MAX_SIZE` (const, 1MB cap)
- `IPLD_MEMORY_BIN` (const, path)
- `shmem_put()` — store blocks in IPLD CAR

**3. File Sampler** 🧬 (DASL hypermorphisms)
- `FileSample` struct (Serialize, Deserialize)
- `SAMPLE_HEAD_LINES`, `SAMPLE_TAIL_LINES`, `SAMPLE_MIDDLE_LINES`, `SAMPLE_CONFORMAL_LINES`
- `sample_file()` — head+tail+middle+conformal sampling
- `compute_entropy()` — Shannon entropy
- `compute_hecke_score()` — Hecke spectral score
- `count_cids()` — detect bafyrei/bafkrei/Qm prefixes

**4. IngestedSubmodule**
- `IngestedSubmodule` struct

**5. Memecache Operations** 🧹
- `handle_memecache_upgrade()` — force-upgrade crate deps
- `count_dependencies()` — count deps in toml::Value
- `handle_memecache_gc()` — clean stale crate versions
- `parse_crate_version()` — parse "crate-1.2.3" directory names
- `dir_size()` — recursive directory size

**6. Scan Index**
- `ScannedFile` struct (Serialize, Deserialize)
- `ScannedGitmodule` struct
- `handle_scan_index()` — multi-source scanner + sampler
- `read_file_list()` — parse text file lists
- `read_gitmodules_file()` — parse via git-config
- `plocate_gitmodules()` — find .gitmodules via plocate
- `read_parquet_index()` — read parquet via python3+pyarrow

### What Remains in `src/main.rs` (646 lines)
- `main()` — CLI dispatcher
- 7 vendoring handlers: `handle_init`, `handle_fetch_upstream`, `handle_rebase`, `handle_releases`,
  `handle_status`, `handle_sync`, `handle_patch`
- `handle_ingest` — submodule registry ingestion (uses utils items)
- `run_workload_analysis`, `run_workload_worktree`
- Stubs: `get_defined_workloads()`, `handle_scan_delta()`

### DASL Architecture Badges
Items that connect to the broader DASL system should be badged in the tile:
- 🧬 **Hecke/Entropy** — `FileSample`, `compute_entropy`, `compute_hecke_score` (hypermorphism spectral analysis)
- 🔗 **IPLD Shmem** — `shmem_put`, `SHMEM_MAX_SIZE`, `IPLD_MEMORY_BIN` (DASL storage layer)
- #️⃣ **CID** — `compute_cid` (content addressing)
- 🧹 **Dependency Graph** — `handle_memecache_gc`, `parse_crate_version`, `dir_size` (crate version management)

### Reference Files
- `src/main.rs` — 646 lines in worktree
- `src/utils.rs` — 1093 lines in worktree
- `src/lib.rs` — 30+ module declarations
- `~/DOCS/VENDORMOD_REFACTOR_MAIN_RS.md` — full documentation
- `~/dasl/plan.org` — search "update jun 7"
- `~/dasl/ipld-car-ipc-shmem-linux/tasks/diagonalize/tile-server/static/diagonalize-tile.html` — tile pattern
