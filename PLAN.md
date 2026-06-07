# cargo-vendormod nora-index Extension Plan

## Goal
Extend cargo-vendormod with a `nora-index` command to store crate metadata as IPLD-compatible DAG-CBOR in the shared memory server, and create a `deep-scanner` tool to apply deep sampling to cached CAR data.

## Accomplished
- ✅ Created `src/nora_index.rs` with IPLD schemas (LocalMirror, NixBuild, PipelightInfo, Vendorsization, PatchesInfo, ConsumersInfo, CrateIndexEntry)
- ✅ Created `src/bin/nora-index.rs` standalone binary with CarStore integration
- ✅ Created `src/bin/deep-scanner.rs` - rewritten to read from shmem cache via CarStore (not disk)
- ✅ Updated `Cargo.toml` with `ipld-car-ipc-shmem-linux` and `memmap2` dependencies  
- ✅ Added `nora_index` module to `src/lib.rs`
- ✅ Created skill docs in `~/projects/n0x-pi/agent/skills/nora-index/SKILL.md`

## DASL Repository Analysis
- Restored `~/dasl/rust/Cargo.toml` (workspace with 12 members)
- `dasl-graph-merged/global_graph.json` contains 1.5M lines: ~585K nodes, ~585K edges
- `dasl-graph-merged/projects.json` lists ~300 projects
- `dasl-graph-merged/upgrade_plan.json` has dependency upgrade recommendations
- `dasl-graph-merged/mirrors.json` has 40 git repository URLs for bare mirrors

## DAG Structure (from src/graph.rs)
The `merge_graphs` function builds:
1. Merges all `graph.json` files from subdirectories
2. Runs petgraph topological sort → `build_order.json`
3. Identifies version mismatches → `upgrade_plan.json`
4. Extracts git sources for mirroring → `mirrors.json`

## Repository DAG Structure (from git_status.txt)
The `~/dasl` directory contains a multi-layer DAG:

### Layer 1: IMPL (Implementation repos)
- `IMPL/atproto-crates/` - atproto DASL
- `IMPL/users/atproto/atproto_repos/` - 75+ atproto-related repos (sorted alphabetically, each is a submodule)
- `IMPL/users/rust/` - Rust workspace members (atrium, bobcoin, cbor4ii, forest, homestar, ipld-nostd, mnem, ref-fvm, rsky, rust-ceramic, rs-ucan, rs-wnfs, szdt)
- `IMPL/jacquard/`, `IMPL/everparse/`, `IMPL/honggfuzz-rs/` - infra tools

### Layer 2: lang (Language implementations)
- `lang/cbor-x/`, `lang/cbor2/`, `lang/cborg/`, `lang/cid/` - CBOR tooling
- `lang/dag-cbor/`, `lang/dag-cbor-meta/` - DAG-CBOR
- `lang/go-dasl/`, `lang/go-ipld-cbor/`, `lang/go-ipld-prime/` - Go implementations
- `lang/iroh/` - Iroh IPLD
- `lang/js-dag-cbor/` - JavaScript
- `lang/python-libipld/`, `lang/python-libipld-meta/` - Python
- `lang/zcbor/` - ZCBOR

### Layer 3: rust workspace
- `rust/ipld-core/`, `rust/n0_dasl/`, `rust/serde_ipld_dagcbor/`, `rust/serde_ipld_dagcbor-escaped/`

### Layer 4: external
- `external/ipfs-docs/`, `external/ipfs/boxo/`, `external/ipfs/go-ipfs-api/`, `external/ipfs/go-libp2p/`, `external/ipfs/helia/`, `external/ipfs/kubo/`
- `external/ipld-car-rs/`, `external/rust-ipld-dagpb/`

## Cleanup Actions
1. Remove `.flake.lock` files from root → move to proper project directories
2. Organize `DOCS/` subdirectories → integrate into proper analysis flow
3. Clean up `deep_scanner_data_*` → use proper cache structure
4. Remove `#file#` backup files in untracked directories

## DMZ Tag 42 Graph
Created `daglm-tests/dmz_tag42_graph.json` - a unified graph of:
- **Binary bytes**: `0xD8` (tag prefix), `0x2A` (CID tag), `0x58` (byte string), `0x81` (array-1)
- **Source functions**: Rust (types::Tag, CBOR_TAGS_CID, cbor4ii), Python (cbor2), Go (cid.NewCidV1), JS (CID.create), Java (encodeTag), C (zcbor)
- **Structure**: CAR header format, CIDv1 Raw codec structure
- **Spec**: CDDL definition for tag42

Transition matrix computed from 9,331 `0xD8→0x2A` transitions out of 173,426 total transitions involving these bytes.

### Hecke Operators (from DMZ_HECKE_ANALYSIS.md)
Key Hecke values:
- p=3: T_p(0xD8) = 6667
- p=7: T_p(0xD8) = 442
- p=17: T_p(0x2A) = 9312 (matches the 0x2A→0x58 transition count!)
- p=67: T_p(0xD8) = 6667

### Fourier Analysis (top modes)
- D8 strongest: k=0 (36293), k=1 (26122), k=255 (26122), k=249 (25677)
- 2A strongest: k=0 (143293), k=50 (119484), k=206 (119484), k=1 (113680)

## DASL to DMZ Graph Architecture
Created `daglm-tests/dasl_to_dmz_graph.json` and `dasl_to_dmz.dot` showing:

### 4-Layer Distance-from-DMZ Structure
1. **Layer 1 (DMZ Core)**: 0xD8 0x2A signature bytes - universal CID encoding center
2. **Layer 2 (Direct Users)**: 6 native implementations all encode to 0xD8 (Rust, Python, Go, JS, Java, C)
3. **Layer 3 (Consumer Crates)**: Workspace crates consuming tag42 implementations
4. **Layer 4 (IMPL Projects)**: Higher-level projects using consumer crates

All Layer 2 implementations are **equidistant** from the DMZ center - no single language is closer, they all converge on the same byte signature.

### Inter-User Relationships (Layer 2 Wall)
The 6 direct users in Layer 2 also have relationships between them:
- **cbor4ii** (Rust) and **serde_ipld_dagcbor** (Rust) are coupled
- **cbor2** (Python/Rust hybrid) bridges to **dag_cbrrr** (Python pure)
- **ipld** (Rust/Go/JS/Java) forms the IPLD cross-language network
- **zcbor** (C) connects to **cbor4ii** via CBOR spec compliance
- All reference the **CDDL specification** for DAG-CBOR structure

### Files to Warm (Task 2 - Multi-Language Sources)
For completing Layer 2 in shmem cache:
```
dasl/rust/serde_ipld_dagcbor-escaped/src/lib.rs     # Layer 1: 42 constant
dasl/rust/serde_ipld_dagcbor-escaped/src/ser.rs    # Layer 2: Tag(42) encoder
dasl/rust/serde_ipld_dagcbor-escaped/src/de.rs      # Layer 2: Tag 42 decoder

dasl/lang/dag-cbor/*.py                            # Layer 2: dag_cbrrr
dasl/lang/go-ipld-prime/cid/cid.go                  # Layer 2: Go CID
dasl/lang/js-dag-cbor/src/index.ts                  # Layer 2: JS ipld-dag-cbor

dasl/lang/java-ipld-cbor/src/main/java/*.java       # Layer 2: Java
dasl/lang/zcbor/cbor_encoding.c                      # Layer 2: C zcbor

### eBPF Monitor for 0xD8 0x2A (Task 8)
Created `daglm-tests/d8_2a_ebpf/` and `daglm-tests/d8_2a_user/` - Rust aya eBPF monitoring:

- **eBPF program**: Scans all 16 GP registers for 0xD8 0x2A byte sequence
- **Userspace loader**: Attaches perf_event program to CPU cycles, opens perf buffer
- **No disk writes**: Only emits hits to userspace via perf buffer
- **Usage**: Build with `cargo build --target bpfel-unknown-none` then run `d8_2a_user` binary
- **Depends on**: `aya` submodule at `/mnt/data1/time-2026/06/03/aya/`

### DMZ Driver (build + test runner)
Created `daglm-tests/dmz_driver.rs` and `run_dmz_capture.rs` to:
1. Build eBPF monitor with `cargo build --target bpfel-unknown-none`
2. Run builds/tests on all 10 projects (6 Layer 2 + 4 Layer 3/4)
3. Capture 0xD8 0x2A hits from perf buffer
4. Record hits to `/home/mdupont/dasl/DOCS/dmz_hits_from_drivers.json`
5. Store results in shmem cache

### Test runner script
Created `daglm-tests/run_tests_with_dmz.sh` to run Layer 2 tests in clean_project:
- Rust: serde_ipld_dagcbor, libipld
- Go: go-ipld-prime
- Python: dag-cbrrr
- JavaScript: js-dag-cbor
- eBPF monitor: captures 0xD8 0x2A hits during test execution

### Note on nora registry
The dasl-testing framework requires nora registry at `http://127.0.0.1:4000/cargo/index`.
To run tests with nora:
```bash
# Start nora (from /mnt/data1/time-2026/05-may/28/nora):
cargo run --release -p nora

# Or bypass with:
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
```