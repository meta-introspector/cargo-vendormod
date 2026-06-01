---
name: libipld-upgrade-fuzzing
description: Set up multi-engine fuzz checking for libipld consumer crates, modeled on the ipld-core fuzzing infrastructure. Covers Honggfuzz, AFL, libFuzzer, fuzzcheck-rs, Bolero, Ziggy, LibAFL, Siderophile, and TSFFS integration with Nix flakes.
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: /home/mdupont/.agents/skills/libipld-upgrade-fuzzing/SKILL.md
---

# libipld Fuzzing Infrastructure

**Purpose**: Provide comprehensive fuzz testing for each libipld consumer crate, using the multi-engine setup proven in `rust/ipld-core`.

## Reference Implementation

The canonical fuzzing setup lives at:
- `/mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core/Makefile`
- `/mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core/fuzz/`
- `/mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core/flakes/fuzzcheck-rs/flake.nix`

## Fuzzing Engines

| Engine | Flake Source | Technique | Best For |
|--------|-------------|-----------|----------|
| **Honggfuzz** | `rust-overlay` (built-in) | Coverage-guided, persistent | General-purpose Rust fuzzing |
| **libFuzzer** (cargo-fuzz) | `./cargo-fuzz` | LLVM SanitizerCoverage | Fast in-process fuzzing |
| **AFL.rs** | `./afl.rs` | AFL++ via Rust wrapper | Complex input formats |
| **fuzzcheck-rs** | `./flakes/fuzzcheck-rs` | Structure-aware, coverage-guided | Data structures, IPLD types |
| **Bolero** | `./flakes/bolero` | Multi-engine fuzzer | Quick smoke tests |
| **Ziggy** | `./ziggy` | CLI-oriented fuzzing | Crate-level black-box tests |
| **LibAFL** | Standalone | Modular, multi-strategy | Advanced fuzzing research |
| **Siderophile** | `./siderophile` | Symbolic analysis | Finding "interesting" inputs |
| **TSFFS** | `./fuzz-tsffs-ipld` | QEMU-based snapshot | No_std / embedded targets |

## State of current fuzz targets

### `rust/ipld-core` (fully set up)

```
rust/ipld-core/
├── fuzz/
│   ├── Cargo.toml
│   ├── corpus/fuzz_target_1/         # 2000+ seed files
│   ├── fuzz_target_honggfuzz.rs      # Dual honggfuzz/libFuzzer target
│   └── build.rs                      # Optional build script
├── fuzz-libafl-ipld/                 # LibAFL standalone
├── fuzz-siderophile-ipld/            # Siderophile analysis
├── fuzz-tsffs-ipld/                  # TSFFS (no_std)
├── fuzz-libfuzzer-ipld/              # Cargo-fuzz
│   └── fuzz/
│       ├── Cargo.toml
│       └── fuzz_targets/
│           └── fuzz_target_1.rs
├── fuzz-fuzzcheck-ipld/              # Fuzzcheck-rs structure-aware
├── fuzz-ziggy-ipld/                  # Ziggy
├── fuzz-bolero-ipld/                 # Bolero
├── flakes/
│   └── fuzzcheck-rs/
│       └── flake.nix                 # Nix flake for fuzzcheck-rs
└── Makefile                          # Full orchestration
```

## Setup Steps for a New Consumer

### 1. Create the fuzz target

```rust
// fuzz/fuzz_target_honggfuzz.rs — works with both libFuzzer and Honggfuzz
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The actual fuzz logic:
    // - Parse `data` as the input format your crate expects
    // - Call the libipld-dependent function
    // - Assert no panics or crashes
    let _ = my_crate::my_libipld_function(data);
});
```

### 2. Create `fuzz/Cargo.toml`

```toml
[package]
name = "consumer-fuzz"
version = "0.1.0"
edition = "2021"
publish = false

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
honggfuzz = { version = "1", optional = true }

[dependencies.my-crate]
path = ".."

[features]
default = ["libfuzzer"]
honggfuzz = ["honggfuzz"]
libfuzzer = []

[[bin]]
name = "fuzz_target_1"
path = "fuzz_target_honggfuzz.rs"
test = false
doc = false
```

### 3. Create seed corpus

```bash
mkdir -p fuzz/corpus/fuzz_target_1
# Add initial seed files — at least one valid input
echo "dummy" > fuzz/corpus/fuzz_target_1/seed1.bin
```

### 4. Add Makefile targets

```makefile
.PHONY: fuzz fuzz-cargo fuzz-honggfuzz fuzz-afl fuzz-fuzzcheck

fuzz: fuzz-cargo

fuzz-cargo:
	nix develop ./cargo-fuzz --command cargo fuzz run fuzz_target_1

fuzz-honggfuzz:
	nix develop .# --command bash -c "cd fuzz && cargo hfuzz run fuzz_target_honggfuzz --features honggfuzz --no-default-features"

fuzz-afl:
	nix develop ./afl.rs --command bash -c "mkdir -p fuzz/afl-in && cp fuzz/corpus/fuzz_target_1/* fuzz/afl-in/ 2>/dev/null; cargo afl fuzz -i fuzz/afl-in -o hfuzz_workspace/fuzz_target_1/afl_out -- ./target/debug/fuzz_target_honggfuzz"

fuzz-fuzzcheck:
	nix develop ./flakes/fuzzcheck-rs --command bash -c "RUSTFLAGS='-C instrument-coverage' cargo fuzzcheck temp_test::tests::my_simple_fuzz_test"
```

### 5. Integrate with Nix

```bash
# Each fuzzing engine has its own flake in rust/ipld-core/flakes/
# Copy the relevant flake.nix for your consumer and adjust paths

# Example: fuzzcheck-rs flake (from rust/ipld-core/flakes/fuzzcheck-rs/flake.nix)
# Uses local git mirror for fuzzcheck-rs source:
# url = "git+file:///mnt/data1/git/github.com/loiclec/fuzzcheck-rs.git?rev=eb076aa14259b5966fcb7458769fff3c453370d0&ref=main"
```

## Makefile Reference (from ipld-core)

```makefile
# Full fuzzing orchestration — copy and adapt for each consumer

fuzz: fuzz-cargo                               # Default: libFuzzer
fuzz-cargo:                                     # Cargo-fuzz (libFuzzer)
fuzz-honggfuzz:                                 # Honggfuzz
fuzz-afl:                                       # AFL.rs
fuzz-bolero:                                    # Bolero
fuzz-ziggy:                                     # Ziggy
fuzz-fuzzcheck:                                 # Fuzzcheck-rs (structure-aware)
fuzz-libafl:                                    # LibAFL
fuzz-siderophile:                               # Siderophile symbolic analysis
fuzz-tsffs:                                     # TSFFS (QEMU-based, no_std)
build:                                          # Build all fuzz targets
rollup:                                         # Generate comprehensive report
```

## Core fuzzing command patterns

```bash
# Honggfuzz
nix develop .# --command bash -c "cd fuzz && RUST_BACKTRACE=1 HFUZZ_RUN_ARGS='-i corpus/fuzz_target_1' cargo hfuzz run fuzz_target_honggfuzz --features honggfuzz --no-default-features"

# AFL
nix develop ./afl.rs --command bash -c "cd fuzz && cargo afl fuzz -i afl-in -o hfuzz_workspace/fuzz_target_1/afl_out -- ./target/debug/fuzz_target_honggfuzz"

# libFuzzer
nix develop ./cargo-fuzz --command cargo fuzz run fuzz_target_1 --no-default-features --features libfuzzer

# Fuzzcheck-rs
nix develop ./flakes/fuzzcheck-rs --command bash -c "cd fuzz && RUSTFLAGS='-C instrument-coverage' cargo fuzzcheck temp_test::tests::my_simple_fuzz_test"

# Bolero
nix develop ./flakes/bolero --command bash -c "cd fuzz && cargo bolero test fuzz_target_1_bolero --features bolero --no-default-features"

# Ziggy
nix develop ./ziggy --command bash -c "cd fuzz && cargo ziggy fuzz fuzz_target_1_ziggy -i ../corpus/fuzz_target_1"

# LibAFL
cd fuzz-libafl-ipld && cargo run --release

# Siderophile
nix develop ./siderophile --command bash -c "cd fuzz-siderophile-ipld && cargo run"
```

## Bootstrap Script

A bootstrap script at `bootstrap-fuzz.sh` generates all 7 fuzzing engine directories for each Rust harness:

```bash
cd /home/mdupont/dasl/dasl-testing/harnesses
bash /home/mdupont/.agents/skills/libipld-upgrade-fuzzing/bootstrap-fuzz.sh
```

For each harness it creates:
- `fuzz/` — Honggfuzz/libFuzzer dual-mode
- `fuzz-libfuzzer/` — Cargo-fuzz (libFuzzer)
- `fuzz-bolero/` — Bolero
- `fuzz-ziggy/` — Ziggy
- `fuzz-fuzzcheck/` — Fuzzcheck-rs (structure-aware)
- `fuzz-libafl/` — LibAFL
- `fuzz-siderophile/` — Siderophile symbolic analysis
