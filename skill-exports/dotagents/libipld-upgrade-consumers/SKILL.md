---
name: libipld-upgrade-consumers
description: Master the end-to-end workflow for discovering libipld consumer repos in the DASL codebase, adding them as targets to the libipld-upgrade harness, and setting up fuzz checking. Covers auto-discovery, git mirror management, target registration, fuzzing integration, and Nix-based testing.
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: /home/mdupont/.agents/skills/libipld-upgrade-consumers/SKILL.md
---

# libipld-upgrade-consumers Skill

**Purpose**: Systematically discover, register, and test all Rust repos in the DASL monorepo that consume `libipld` crates, using the `libipld-upgrade` harness with local git mirrors, plus fuzz checking for each consumer.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Discovery Pipeline                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────┐    ┌─────────────────┐    ┌───────────────┐  │
│  │ git submodule    │───▶│ grep libipld in │───▶│ Derive mirror │  │
│  │ foreach + remote │    │ Cargo.toml      │    │ path & target │  │
│  └──────────────────┘    └─────────────────┘    └───────┬───────┘  │
│                                                          │          │
│                                                          ▼          │
│  ┌──────────────────┐    ┌─────────────────┐    ┌───────────────┐  │
│  │ Add to           │◀───│ Create git      │◀───│ Generate      │  │
│  │ targets.nix/json │    │ mirror if miss  │    │ target config │  │
│  └──────────────────┘    └─────────────────┘    └───────────────┘  │
│                                                          │          │
│                                                          ▼          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                     Run Full Matrix                           │   │
│  │  nix run /path/to/libipld-upgrade#test-all                    │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Prerequisites

- **DASL monorepo** at `/home/mdupont/dasl`
- **libipld-upgrade harness** at `/home/mdupont/dasl/IMPL/libipld-upgrade`
- **Git mirror base** at `/mnt/data1/git/github.com/`
- **Shim workspace** at `/mnt/data1/time-2026/02-february/22/dasl/IMPL/users/atproto/atproto_repos/ipld_libipld`
- **ipld-core fuzzing reference** at `/mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core`

## Step 1: Auto-Discover libipld Consumer Submodules

Use the provided discovery script:

```bash
cd /home/mdupont/dasl
bash /home/mdupont/.agents/skills/libipld-upgrade-consumers/discover-consumers.sh
```

This script:
1. Iterates all git submodules via `git submodule foreach`
2. Checks each for `libipld` references in `Cargo.toml`
3. Derives the git mirror path in `/mnt/data1/git/`
4. Checks if the mirror exists; creates it if missing
5. Generates target entries for `targets.nix` and `targets.json`
6. Outputs a report: which targets are ready, which need mirrors, and which need shim compatibility fixes

### Manual Discovery

```bash
# List all submodules with their remote URLs
cd /home/mdupont/dasl
git submodule foreach --quiet 'echo "$path|$(git remote get-url origin)"' > submodules.txt

# Find submodules with libipld in Cargo.toml
while IFS='|' read -r path url; do
  if [ -f "$path/Cargo.toml" ] && grep -q 'libipld' "$path/Cargo.toml" 2>/dev/null; then
    echo "$path|$url"
  fi
done < submodules.txt
```

## Step 2: Git Mirror Management

### How mirror paths are derived

Given a git URL `https://github.com/owner/repo.git`:
- Mirror path: `/mnt/data1/git/github.com/owner_repo.git`
- Target name: `owner_repo` (used in `targets.nix` and `targets.json`)

### Create missing mirrors

```bash
MIRROR_BASE=/mnt/data1/git/github.com
url=https://github.com/owner/repo.git
owner_repo="${url#https://github.com/}"
owner_repo="${owner_repo%.git}"
mirror_dir="${owner_repo/\//_}"

if [ ! -d "$MIRROR_BASE/${mirror_dir}.git" ]; then
  git clone --mirror "$url" "$MIRROR_BASE/${mirror_dir}.git"
fi
```

### Derive the git file URL for the harness

```bash
owner_repo="owner_repo"
rev="abc123def"  # Full commit hash
ref="main"
url="git+file:///mnt/data1/git/github.com/${owner_repo}.git?rev=${rev}&ref=${ref}"
```

### Getting the current HEAD revision

```bash
cd /home/mdupont/dasl/IMPL/users/atproto/atproto_repos/owner_repo
git rev-parse HEAD
```

## Step 3: Add Targets to libipld-upgrade

### targets.nix

```nix
  "owner_repo" = mkTarget {
    name = "owner_repo";
    host = "github.com";
    owner = "owner";
    repo = "repo";
    rev = "abc123def";
    ref = "main";
    patchMode = "source-patch";   # or "cargo-patch"
    testCmd = "cargo test --workspace";
  };
```

### targets.json

```json
  "owner_repo": {
    "name": "owner_repo",
    "host": "github.com",
    "owner": "owner",
    "repo": "repo",
    "rev": "abc123def",
    "ref": "main",
    "patchMode": "source-patch",
    "testCmd": "cargo test --workspace",
    "shimPath": "/mnt/data1/time-2026/02-february/22/dasl/IMPL/users/atproto/atproto_repos/ipld_libipld",
    "url": "git+file:///mnt/data1/git/github.com/owner_repo.git?rev=abc123def&ref=main"
  }
```

### Makefile

```makefile
upgrade-owner_repo:
	$(RUN_ENV) $(NIX) run $(FLAKE)#upgrade-owner_repo
```

## Step 4: Patch Mode Selection

| Mode | When to use |
|------|-------------|
| `cargo-patch` | Repos using standard crates.io `libipld = "^0.16"` style deps. Leaves deps intact, appends `[patch.crates-io]` section. |
| `source-patch` | Repos with git or path dependencies to old libipld crates. Rewrites dep declarations directly. Removes Cargo.lock. |

### Determining patch mode

```bash
# Check the kind of dependencies in the repo
cd /path/to/repo
grep -r 'libipld' --include='Cargo.toml' | grep -E 'git|path' && echo "source-patch" || echo "cargo-patch"
```

## Step 5: Run and Verify

```bash
cd /home/mdupont/dasl/IMPL/libipld-upgrade

# Test a single target
nix run .#upgrade-owner_repo

# Test all targets
nix run .#test-all
```

## Step 6: Fuzz Checking Setup

For each consumer, set up fuzz checking modeled on `rust/ipld-core`.

### Reference structure (ipld-core)

```
rust/ipld-core/
├── fuzz/                              # Main fuzz directory
│   ├── Cargo.toml                     # Fuzz workspace manifest
│   ├── corpus/                        # Seed corpus
│   │   └── fuzz_target_1/
│   ├── fuzz_target_honggfuzz.rs       # Honggfuzz target
│   └── ...
├── fuzz-libafl-ipld/                  # LibAFL fuzzer
├── fuzz-siderophile-ipld/             # Siderophile symbolic analysis
├── fuzz-tsffs-ipld/                   # TSFFS fuzzer (QEMU-based)
├── fuzz-libfuzzer-ipld/               # Cargo-fuzz (libFuzzer)
│   └── fuzz/
│       ├── Cargo.toml
│       └── fuzz_targets/
│           └── fuzz_target_1.rs
├── fuzz-fuzzcheck-ipld/               # Fuzzcheck-rs (structure-aware)
├── fuzz-ziggy-ipld/                   # Ziggy fuzzer
├── fuzz-bolero-ipld/                  # Bolero fuzzer
├── flakes/                            # Nix flakes for each fuzzing engine
│   ├── fuzzcheck-rs/
│   │   └── flake.nix
│   └── ...
└── Makefile                           # Fuzzing orchestration
```

### Key fuzzing targets to set up for each consumer:

| Fuzzer | Directory pattern | Flake path |
|--------|------------------|------------|
| Honggfuzz | `fuzz/Cargo.toml` + `fuzz/fuzz_target_honggfuzz.rs` | `.` (uses `rust-overlay`) |
| Cargo-fuzz (libFuzzer) | `fuzz-libfuzzer-ipld/` | `./cargo-fuzz` |
| AFL | `fuzz/Cargo.toml` (afl feature) | `./afl.rs` |
| Fuzzcheck-rs | `fuzz-fuzzcheck-ipld/` | `./flakes/fuzzcheck-rs` |
| Bolero | `fuzz-bolero-ipld/` | `./flakes/bolero` |
| Ziggy | `fuzz-ziggy-ipld/` | `./ziggy` |
| LibAFL | `fuzz-libafl-ipld/` | N/A (standalone) |
| Siderophile | `fuzz-siderophile-ipld/` | `./siderophile` |
| TSFFS | `fuzz-tsffs-ipld/` | `./fuzz-tsffs-ipld` |

### Minimal fuzz setup for a consumer

1. **Create fuzz directory**:
```bash
mkdir -p lib/CARGO_CRATE/fuzz/corpus/fuzz_target_1
```

2. **Create fuzz/Cargo.toml**:
```toml
[package]
name = "consumer-fuzz"
version = "0.1.0"
edition = "2021"

[dependencies]
libfuzzer-sys = "0.4"
honggfuzz = { version = "1", optional = true }

[dependencies.consumer-crate]
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

3. **Create fuzz target** (`fuzz/fuzz_target_honggfuzz.rs`):
```rust
// Works with both libFuzzer and Honggfuzz
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the consumer's libipld-dependent code
    let _ = consumer_crate::some_libipld_function(data);
});
```

### Makefile targets for fuzzing

```makefile
# Add to consumer's Makefile or a makefile fragment

fuzz-cargo:
	nix develop ./cargo-fuzz --command cargo fuzz run fuzz_target_1

fuzz-honggfuzz:
	nix develop .# --command bash -c "cd fuzz && cargo hfuzz run fuzz_target_honggfuzz --features honggfuzz --no-default-features"

fuzz-afl:
	nix develop ./afl.rs --command bash -c "cd fuzz && cargo afl fuzz -i afl-in -o hfuzz_workspace/fuzz_target_1/afl_out -- ./target/debug/fuzz_target_1"

fuzz-fuzzcheck:
	nix develop ./flakes/fuzzcheck-rs --command bash -c "cd fuzz && RUSTFLAGS='-C instrument-coverage' cargo fuzzcheck temp_test::tests::my_simple_fuzz_test"
```

## Step 7: Run Full Matrix with Parallel Targets

```bash
cd /home/mdupont/dasl/IMPL/libipld-upgrade

# Sequential run
nix run .#test-all

# Parallel run with GNU parallel
nix run .#show-targets | jq -r 'keys[]' | parallel -j4 '
  echo "=== Running: {} ==="
  nix run .#upgrade-{}
  echo
'
```

## Known Consumer Repositories

These submodules in DASL reference `libipld` and are candidates for the upgrade harness:

| Submodule Path | GitHub Repo | Patch Mode | Mirror Status |
|----------------|-------------|------------|---------------|
| `IMPL/users/atproto/atproto_repos/Actyx_cbor-data` | Actyx/cbor-data | source-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/MarshalX_python-libipld` | MarshalX/python-libipld | source-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/aDotInTheVoid_triphosphate` | aDotInTheVoid/triphosphate | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/aditsachde_hamt-rs` | aditsachde/hamt-rs | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/consensus-shipyard_ipc` | consensus-shipyard/ipc | source-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/consensus-shipyard_ipc-ipld-resolver` | consensus-shipyard/ipc-ipld-resolver | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/dignifiedquire_iroh-cbor-example` | dignifiedquire/iroh-cbor-example | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/fleek-network_ursa` | fleek-network/ursa | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/hdevalence_mst` | hdevalence/mst | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/ipfs-rust_ipld-block-builder` | ipfs-rust/ipld-block-builder | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/ipfs-rust_libipld-collections` | ipfs-rust/libipld-collections | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/ipld_libipld` | ipld/libipld (actually junky code) | cargo-patch | ✅ Exists |
| `IMPL/users/atproto/atproto_repos/recallnet_ipc` | recallnet/ipc | source-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/rklaehn_cbor-tag-index` | rklaehn/cbor-tag-index | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/rklaehn_libipld-raw-cbor` | rklaehn/libipld-raw-cbor | cargo-patch | ❌ Missing |
| `IMPL/users/atproto/atproto_repos/timoCasti_dep-libipld` | timoCasti/dep-libipld | source-patch | ✅ Exists |
| `IMPL/users/atproto/atproto_repos/timoCasti_libipld` | timoCasti/libipld | cargo-patch | ❌ Missing |
| `external/ipld-car-rs` | meta-introspector/ipld-car | cargo-patch | ❌ Missing |
| `lang/python-libipld` | MarshalX/python-libipld | source-patch | ❌ Missing |
| `lang/python-libipld-meta` | meta-introspector/python-libipld | source-patch | ❌ Missing |
| `rust/ipld-core` | ipld/rust-ipld-core | source-patch | ✅ Exists (shim source) |

## Local Monorepo Files (non-submodules)

These files within the DASL monorepo reference libipld but are NOT submodules. They need manual patching or the harness must be adapted for local paths:

| File | Description |
|------|-------------|
| `external/dasl-meta/3rd-party/dasl-testing/harnesses/libipld/` | Test harnesses using libipld |
| `lang/dag-cbrrr/codec-fixtures/rust/` | Codec fixtures (part of dag-cbrrr submodule) |
| `rust/n0_dasl/3rd-party/dasl-testing/harnesses/libipld/` | Test harnesses in n0_dasl submodule |
| `IMPL/atproto-crates/crates/atproto-dasl/tests/dasl-testing/harnesses/libipld/` | Test harnesses in atproto-crates |
| `IMPL/crate2nix-zos/vendor/rust-ipfs/` | Vendored rust-ipfs (part of crate2nix-zos submodule) |

## Troubleshooting

### Mirror doesn't exist
```bash
# Check if mirror exists
ls /mnt/data1/git/github.com/ | grep owner_repo

# Create it if missing
git clone --mirror https://github.com/owner/repo.git /mnt/data1/git/github.com/owner_repo.git
```

### Cargo metadata fails after patching
- Check if the shim workspace is properly set up
- Try the other patch mode
- Remove Cargo.lock and try again
- Check for yanked dependencies (common with old libipld-core)

### Test failures
- Check if the repo has workspace-level vs crate-level tests
- Override `TEST_CMD` for the specific target
- Try `cargo check` instead of `cargo test` first

## Commands Quick Reference

```bash
# Discover consumers
cd /home/mdupont/dasl && bash discover-consumers.sh

# Create missing mirrors
git clone --mirror https://github.com/owner/repo.git /mnt/data1/git/github.com/owner_repo.git

# Get HEAD revision of a submodule
git -C IMPL/users/atproto/atproto_repos/owner_repo rev-parse HEAD

# Run upgrade harness
cd /home/mdupont/dasl/IMPL/libipld-upgrade
nix run .#upgrade-owner_repo
nix run .#test-all

# Run fuzzing
# (in the consumer's directory)
make fuzz-cargo
make fuzz-honggfuzz
```

## Files

```
libipld-upgrade-consumers/
├── SKILL.md                        # This file
└── discover-consumers.sh           # Auto-discovery script
```
