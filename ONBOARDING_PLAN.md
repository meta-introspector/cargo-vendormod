# Complex Workspace Onboarding Plan

## Overview
This plan outlines how to onboard a complex nested workspace (like Solana) to cargo-vendormod, including recursive crate discovery, forking, and patching.

## Phase 1: Workspace Analysis

### Step 1: Recursive Crate Discovery
**Goal**: Identify all crates in the workspace hierarchy

**Actions**:
- Use `cargo metadata` to get workspace structure
- Traverse all workspace members recursively
- Identify both direct and transitive dependencies
- Build complete dependency graph

**Tools**:
```bash
# Get workspace metadata
cargo metadata --format-version 1 --no-deps > workspace_metadata.json

# Use cargo-vendormod to analyze
cargo vendormod analyze-workspace --input workspace_metadata.json --output workspace_analysis.json
```

**Expected Output**:
- Complete list of all crates (50-100+ for Solana)
- Dependency graph visualization
- Crate classification (binaries, libraries, tests)

### Step 2: Dependency Resolution
**Goal**: Resolve all dependencies including git dependencies

**Actions**:
- Parse all Cargo.toml files recursively
- Identify git dependencies that need vendoring
- Resolve version conflicts
- Create dependency resolution report

**Code Reference**:
```rust
// From global_dep_graph.rs
let mut builder = GlobalDependencyGraphBuilder::new();
builder.add_workspace(&workspace_path)?;
builder.resolve_dependencies()?;
let graph = builder.build();
```

## Phase 2: Forking Strategy

### Step 3: Repository Forking
**Goal**: Create local forks of all git dependencies

**Actions**:
- For each git dependency:
  - Fork repository to local git host
  - Clone to submodules directory
  - Set up bare mirror for rebasing
  - Configure dual remotes (upstream + bare)

**Implementation**:
```rust
// From vendoring.rs
let action = RepoAction {
    repo_url: dependency.url.clone(),
    submodule_path: submodules_dir.join(&repo_name),
    mirrors_path: bare_mirrors_dir.to_path_buf(),
    target_branch: target_branch.to_string(),
    // ... other fields
};

add_or_update_submodule(&ctx, &action)?;
```

### Step 4: Submodule Organization
**Goal**: Organize submodules for efficient management

**Structure**:
```
submodules/
├── crate-name-1/
│   ├── .git
│   ├── Cargo.toml
│   └── src/
├── crate-name-2/
│   ├── .git
│   ├── Cargo.toml
│   └── src/
└── ...
```

**Configuration**:
- Each submodule gets dual remotes:
  - `upstream`: Original repository
  - `bare`: Local bare mirror for rebasing

## Phase 3: Patching System

### Step 5: Patch Generation
**Goal**: Generate patches for local modifications

**Actions**:
- For each submodule:
  - Compare with upstream
  - Generate patch files
  - Store in patches directory
  - Update Cargo.toml with patch entries

**Implementation**:
```rust
// From patch.rs
let patches = generate_patches_for_workspace(&ctx)?;
apply_patches_to_cargo_toml(&workspace_path, &patches)?;
```

### Step 6: Patch Application
**Goal**: Apply patches systematically

**Patch Structure**:
```
patches/
├── crate-name-1/
│   ├── 0001-local-modifications.patch
│   └── 0002-feature-addition.patch
├── crate-name-2/
│   └── 0001-bugfix.patch
└── ...
```

**Cargo.toml Updates**:
```toml
[patch.crates-io]
crate-name-1 = { path = "submodules/crate-name-1" }
crate-name-2 = { path = "submodules/crate-name-2" }
```

## Phase 4: Continuous Synchronization

### Step 7: Upstream Sync
**Goal**: Keep forks updated with upstream changes

**Workflow**:
```bash
# Fetch latest upstream changes
cargo vendormod fetch-upstream --all

# Rebase local changes
cargo vendormod rebase --all

# Update dependency versions
cargo vendormod update-versions
```

### Step 8: Conflict Resolution
**Goal**: Handle merge conflicts systematically

**Strategy**:
- Automated conflict detection
- Manual resolution workflow
- Conflict tracking database
- Rollback capability

## Implementation Checklist

- [ ] Workspace analysis module
- [ ] Recursive crate discovery
- [ ] Dependency graph builder
- [ ] Forking automation
- [ ] Submodule management
- [ ] Patch generation system
- [ ] Continuous sync mechanism
- [ ] Conflict resolution tools
- [ ] Monitoring and reporting

## Risk Assessment

**High Risk Areas**:
1. **Dependency Hell**: Complex version conflicts in large workspaces
2. **Git Conflicts**: Rebase conflicts during upstream sync
3. **Build Breakage**: Patches causing compilation failures
4. **Performance**: Slow operations on large workspaces

**Mitigation Strategies**:
- Incremental onboarding (start with core crates)
- Comprehensive testing at each step
- Dry-run modes for all operations
- Rollback capability for failed operations

## Success Metrics

**Phase 1 (Analysis)**:
- ✅ All crates discovered and cataloged
- ✅ Complete dependency graph generated
- ✅ No missing dependencies

**Phase 2 (Forking)**:
- ✅ All git dependencies forked locally
- ✅ Submodules properly organized
- ✅ Dual remotes configured correctly

**Phase 3 (Patching)**:
- ✅ All local modifications captured as patches
- ✅ Cargo.toml patch sections working
- ✅ Build succeeds with patched dependencies

**Phase 4 (Sync)**:
- ✅ Upstream sync working reliably
- ✅ Rebase conflicts handled properly
- ✅ Continuous integration pipeline established

## Timeline Estimate

| Phase | Duration | Key Milestones |
|-------|----------|----------------|
| 1. Analysis | 1-2 weeks | Complete workspace map, dependency graph |
| 2. Forking | 2-3 weeks | All dependencies forked and organized |
| 3. Patching | 1-2 weeks | All patches generated and applied |
| 4. Sync | Ongoing | Automated sync pipeline established |

Total initial onboarding: **4-7 weeks** (depending on workspace complexity)