# libipld-upgrade

Master the libipld-upgrade harness for systematically upgrading Rust repositories from old libipld crates to a shim workspace. Use for patching dependencies, testing compatibility, and managing git mirror-based upgrade workflows.

## Instructions

# libipld-upgrade Harness

**Purpose**: Systematically upgrade Rust repositories that depend on old `libipld` crates to use a local shim workspace, with automated patching, building, and testing.

## Quick Start

```bash
# Enter development shell
nix develop /home/mdupont/dasl/IMPL/libipld-upgrade

# Show all targets
nix run .#show-targets

# Run a single target
nix run .#upgrade-ipld_libipld

# Run full test matrix
nix run .#test-all
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  libipld-upgrade Flake                                    │
├─────────────────────────────────────────────────────────────┤
│  flake.nix          - Nix flake with target definitions      │
│  targets.nix        - Target repos (Nix DSL)                 │
│  targets.json       - Target repos (JSON, for runtime)       │
│  scripts/run-target.sh     - Orchestration                  │
│  scripts/patch-libipld-consumer.sh - Cargo.toml patching    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Git Mirror Base (/mnt/data1/git)                          │
│  ├── github.com/ipld/libipld.git                           │
│  ├── github.com/ipld/serde_ipld_dagcbor.git                │
│  └── github.com/timoCasti/dep-libipld.git                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Shim Workspace (default: /mnt/data1/.../ipld_libipld)      │
│  ├── Cargo.toml                                            │
│  ├── core/                - libipld-core                    │
│  ├── dag-cbor/            - libipld-cbor                     │
│  ├── dag-cbor-derive/     - libipld-cbor-derive              │
│  ├── dag-json/            - libipld-json                     │
│  ├── dag-pb/              - libipld-dag-pb                   │
│  └── macro/               - libipld-macro                    │
└─────────────────────────────────────────────────────────────┘
```

## Workflow

Each target test performs:

1. **Clone** from local git mirror URL
2. **Checkout** specific revision
3. **Patch** Cargo.toml files (rewrite libipld dependencies to shim paths)
4. **Validate** with `cargo metadata --no-deps`
5. **Test** with target-specific command (default: `cargo test --workspace`)
6. **Cleanup** temporary workspace

## Target Definitions

Targets are defined in `targets.nix` and `targets.json`:

| Target | Repository | Patch Mode | Status |
|--------|-----------|------------|--------|
| `ipld_libipld` | github.com/ipld/libipld | cargo-patch | ⚠️ Yanked deps |
| `timoCasti_dep-libipld` | github.com/timoCasti/dep-libipld | source-patch | ⚠️ Version mismatch |
| `ipld_serde_ipld_dagcbor` | github.com/ipld/serde_ipld_dagcbor | cargo-patch | ✅ Ready |

## Patch Modes

### cargo-patch
- Leaves existing dependency declarations intact
- Appends `[patch.crates-io]` section to Cargo.toml
- Best for: Crates using standard crates.io dependencies

```toml
[patch.crates-io]
libipld = { path = "/path/to/shim" }
libipld-core = { path = "/path/to/shim/core" }
# ... etc
```

### source-patch
- Rewrites `libipld*` dependency declarations directly
- Replaces version strings with explicit local `path` entries
- Removes Cargo.lock to force regeneration
- Best for: Repos with git or path dependencies to old libipld crates

## Commands

### Nix Flake Commands

```bash
# Enter development shell
nix develop

# Show all target definitions
nix run .#show-targets

# Run individual target
nix run .#upgrade-ipld_libipld
nix run .#upgrade-timoCasti_dep-libipld
nix run .#upgrade-ipld_serde_ipld_dagcbor

# Run all targets
nix run .#test-all

# Override shim path
SHIM_PATH=/abs/path/to/shim nix run .#upgrade-ipld_libipld
```

### Makefile Commands

```bash
make shell              # Enter dev shell
make show-targets       # Print target metadata
make test-all           # Run full matrix
make upgrade-ipld_libipld
```

## Adding New Targets

1. **Add git mirror** (if not already in `/mnt/data1/git`):
   ```bash
   git clone https://github.com/owner/repo.git /mnt/data1/git/github.com/owner/repo.git
   ```

2. **Edit targets.nix**:
   ```nix
   my_new_target = mkTarget {
     name = "my_new_target";
     host = "github.com";
     owner = "owner";
     repo = "repo";
     rev = "abc123";
     ref = "main";
     patchMode = "cargo-patch";  # or "source-patch"
     testCmd = "cargo test --workspace";
     shimPath = "/path/to/shim";
   };
   ```

3. **Update targets.json** with matching definition

4. **Update Makefile** with new target command

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SHIM_PATH` | `/mnt/data1/time-2026/02-february/22/dasl/IMPL/users/atproto/atproto_repos/ipld_libipld` | Path to shim workspace |
| `PATCH_MODE` | `cargo-patch` | Patch mode to use |
| `TEST_CMD` | `cargo test --workspace` | Test command to run |

### Target Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `name` | string | required | Unique target identifier |
| `host` | string | required | Git host (e.g., github.com) |
| `owner` | string | required | Repository owner |
| `repo` | string | required | Repository name |
| `rev` | string | required | Git commit hash |
| `ref` | string | `main` | Git branch/reference |
| `patchMode` | string | `cargo-patch` | `cargo-patch` or `source-patch` |
| `testCmd` | string | `cargo test --workspace` | Command to run after patching |
| `shimPath` | string | default shim | Path to shim workspace |

## Troubleshooting

### Common Issues

**Yanked dependency (core2 = "^0.4")**
- The old libipld-core has a yanked `core2` dependency
- Solution: The shim workspace uses a vendored copy: `core2 = { path = "../vendor/core2-shim" }`
- Ensure the shim is properly set up with vendored dependencies

**Version mismatch**
- Target requires `libipld-core = "^0.15.0"` but shim provides `0.16.0`
- Solution: Use `source-patch` mode which ignores version constraints, or update shim versions

**Git mirror missing**
- Error: repository not found at `/mnt/data1/git/github.com/owner/repo.git`
- Solution: Clone the repository to the mirror base first

**Cargo.lock conflicts**
- The harness removes Cargo.lock by default
- If this causes issues, modify the patch script to preserve it

### Debug Mode

Add `set -x` at the top of scripts for verbose output:
```bash
# Run with debug
bash -x scripts/run-target.sh --target ipld_libipld --targets-file targets.json
```

## Script Reference

### run-target.sh

Orchestrates the upgrade process for a single target.

**Arguments**:
- `--target NAME` - Target name to run
- `--targets-file FILE` - Path to targets.json

**Environment**:
- `TARGET_NAME`, `TARGET_URL`, `TARGET_REV`, `TARGET_REF`
- `PATCH_MODE`, `TEST_CMD`, `SHIM_PATH`

### patch-libipld-consumer.sh

Patches all Cargo.toml files in a repository to use the shim workspace.

**Handles these crates**:
- `libipld`
- `libipld-core`
- `libipld-cbor`
- `libipld-json`
- `libipld-macro`
- `libipld-cbor-derive`

**Behavior**:
- Finds all Cargo.toml files (excluding target/ and .git/)
- Rewrites string-style dependencies: `libipld = "0.16.0"` → `libipld = { path = "/shim" }`
- Rewrites inline table dependencies with existing paths
- Adds `[patch.crates-io]` section in cargo-patch mode
- Removes Cargo.lock

## Flake Structure

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      {
        apps = {
          upgrade-{name} = # Per-target apps
          test-all = # Runs all targets
          show-targets = # Shows target metadata
        };
        devShells.default = # Development shell with cargo, git, jq, etc.
      });
}
```

## Files

```
libipld-upgrade/
├── flake.nix            # Main flake definition
├── flake.lock           # Flake lock file
├── targets.nix          # Target definitions (Nix)
├── targets.json         # Target definitions (JSON)
├── Makefile             # Convenience commands
├── README.md            # User documentation
├── ANALYSIS.md          # Detailed analysis
└── scripts/
    ├── run-target.sh            # Main orchestration
    └── patch-libipld-consumer.sh # Cargo.toml patching
```

## Best Practices

1. **Always use git mirrors** - Ensures offline, repeatable testing
2. **Pin exact revisions** - Use full commit hashes, not branches
3. **Test incrementally** - Run individual targets before full matrix
4. **Check ANALYSIS.md** - Contains current status and known issues
5. **Use source-patch for git deps** - More reliable for repos with git/path dependencies
6. **Override SHIM_PATH for testing** - Test with different shim versions

## Future Enhancements

- [ ] Add result collection for `test-all` (JSON output)
- [ ] Make mirror base configurable via flake inputs
- [ ] Add better error messages for common failures
- [ ] Add dry-run mode
- [ ] Automate adding targets from dependency scans
- [ ] Support custom test commands per-target
- [ ] Add pre-patch hooks for unusual dependency layouts
- [ ] Add timing metrics

## Examples

- Run the skill workflow as documented in the source.
