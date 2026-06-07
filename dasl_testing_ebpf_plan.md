# DASL Testing eBPF Monitoring Plan

## Goal

Build a comprehensive testing infrastructure that runs all DASL projects with eBPF monitoring to capture 0xD8 0x2A (CBOR tag 42 / CID signature) occurrences in register values during execution.

## Instructions

- Create eBPF-based perf_event monitor to detect 0xD8 0x2A byte sequence in all 16 GP registers
- Integrate with dasl-testing framework's fuzzing/round_robin infrastructure
- Record hits to shmem/IPFS DAG-CBOR format
- Build and test all Layer 2 direct implementations (Rust, Go, Python, JS, Java, C)

## Discoveries

- The dasl-testing framework at `/mnt/data1/time-2026/02-february/22/dasl/dasl-testing/` has a comprehensive Makefile with fuzzing targets for all languages
- It requires nora cargo registry at `http://127.0.0.1:4000/cargo/index` 
- nora source code is at `/mnt/data1/time-2026/05-may/28/nora/`
- nora config exists at `/mnt/data1/nora/config/nora.toml`
- dasl-testing harness for n0_dasl uses `dasl = "0.2.0"` which needs nora registry
- The eBPF code uses aya-bpf but the cargo target `bpfel-unknown-none` may not be installed

## Accomplished

- ✅ Created `daglm-tests/d8_2a_ebpf/` - eBPF perf_event program (scans registers for 0xD8 0x2A)
- ✅ Created `daglm-tests/d8_2a_user/` - Userspace loader with perf buffer reader
- ✅ Created `daglm-tests/d8_2a_ebpf/Cargo.toml` - Points to local aya submodule at `/mnt/data1/time-2026/06/03/aya/`
- ✅ Created `daglm-tests/run_dasl_testing_with_dmz.sh` - Script to orchestrate testing
- ✅ Created `daglm-tests/test_DMX_inputs.txt` - Test inputs containing CBOR tag 42
- ✅ Analyzed dasl-testing framework structure (3 Rust harnesses + Go + Python + JS)
- ✅ Identified nora registry requirement
- ❌ nora server not yet started/running
- ❌ eBPF monitor not yet built/tested
- ⏳ dasl-testing builds failing due to missing nora registry

## Relevant files / directories

### daglm-tests/ (created)
- `d8_2a_ebpf/src/lib.rs` - eBPF program
- `d8_2a_ebpf/Cargo.toml` - eBPF Cargo manifest
- `d8_2a_user/src/main.rs` - Userspace loader
- `d8_2a_user/Cargo.toml` - Userspace Cargo manifest
- `run_dasl_testing_with_dmz.sh` - Test runner script
- `test_DMX_inputs.txt` - CBOR test inputs
- `start_nora.md` - Instructions for starting nora
- `dmz_tag42_graph.json`, `dasl_to_dmz_graph.json`, `dasl_to_dmz.dot` - Graph definitions

### dasl-testing (existing)
- `/mnt/data1/time-2026/02-february/22/dasl/dasl-testing/` - Main testing framework
- `/mnt/data1/time-2026/02-february/22/dasl/dasl-testing/harnesses/n0_dasl/` - Rust harness with round_robin
- `/mnt/data1/time-2026/02-february/22/dasl/dasl-testing/.cargo/config.toml` - Points to nora registry

### nora (registry)
- `/mnt/data1/time-2026/05-may/28/nora/` - Nora source code
- `/mnt/data1/nora/config/nora.toml` - Config for localhost:4000
- `/mnt/data1/time-2026/05-may/28/nora/nora-registry/Cargo.toml` - Nora registry package

## Next Steps

1. Start nora cargo registry server
2. Build and test eBPF monitor
3. Run dasl-testing fuzz targets with eBPF monitoring
4. Collect and analyze 0xD8 0x2A occurrences in register values