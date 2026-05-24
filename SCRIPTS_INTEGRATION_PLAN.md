# 🔧 Scripts Integration Plan
# Cargo-Rail Workflow Scripts Integration

## 📋 Executive Summary

**Discovery:** Found comprehensive workflow scripts in `workload/scripts/` from cargo-rail
**Status:** Analysis complete, integration planned
**Priority:** High - These scripts provide production-ready workflows

## 🔍 Scripts Inventory

### Found in `workload/scripts/`

| Script | Purpose | Status |
|--------|---------|--------|
| `minimal_workflow.sh` | Basic vendoring workflow | ✅ Available |
| `onboard.sh` | Universal onboarding (git/crate/workdir) | ✅ Available |
| `add_flake_nix.sh` | Recursive flake.nix generation | ✅ Available |
| `detect_fork_upstream.sh` | Fork upstream detection | ✅ Available |
| `onboard_fork.sh` | Fork-specific onboarding | ✅ Available |

### Cargo-Rail Scripts (for reference)

| Script | Purpose | Location |
|--------|---------|----------|
| `test.sh` | Test suite runner | `workload/workspaces/cargo-rail/scripts/test/` |
| `pre-push.sh` | Pre-push hook | `workload/workspaces/cargo-rail/scripts/ci/` |
| `check.sh` | Code quality checks | `workload/workspaces/cargo-rail/scripts/check/` |

## 🎯 Integration Strategy

### Phase 1: Immediate Integration (Current)

**Goal:** Integrate existing scripts with minimal modification

#### 1. Create Scripts Directory

```bash
mkdir -p scripts
```

#### 2. Symlink Existing Scripts

```bash
# Create symlinks to cargo-rail scripts
ln -s ../workload/scripts/minimal_workflow.sh scripts/
ln -s ../workload/scripts/onboard.sh scripts/
ln -s ../workload/scripts/add_flake_nix.sh scripts/
ln -s ../workload/scripts/detect_fork_upstream.sh scripts/
ln -s ../workload/scripts/onboard_fork.sh scripts/
```

#### 3. Update Documentation

Add to `QUICK_START_GUIDE.md`:
```markdown
### Available Scripts

**Location:** `scripts/`

1. **minimal_workflow.sh** - Basic vendoring workflow
   ```bash
   ./scripts/minimal_workflow.sh workload/workspaces/cargo-rail
   ```

2. **onboard.sh** - Universal onboarding
   ```bash
   ./scripts/onboard.sh --git-repo https://github.com/example/project
   ```

3. **add_flake_nix.sh** - Flake.nix generation
   ```bash
   ./scripts/add_flake_nix.sh /path/to/workspace
   ```
```

#### 4. Test Scripts

```bash
# Test minimal workflow
./scripts/minimal_workflow.sh workload/workspaces/cargo-rail --dry-run

# Test onboarding
./scripts/onboard.sh --workdir workload/workspaces/cargo-rail --dry-run

# Test flake generation
./scripts/add_flake_nix.sh workload/workspaces/cargo-rail --dry-run
```

### Phase 2: Enhanced Integration

**Goal:** Integrate scripts with our topological sorting pipeline

#### 1. Create Wrapper Scripts

**File:** `scripts/process_with_topological.sh`

```bash
#!/bin/bash
# Process workspace with topological sorting and cargo-rail workflows

set -e
set -u

WORKSPACE_PATH=${1:-.
USAGE="Usage: $0 [workspace-path] [options]

Options:
  --with-vendoring    Run vendoring workflow
  --with-onboarding   Run onboarding workflow
  --with-flakes      Generate flake.nix files
  --dry-run          Dry run mode
  --help             Show this help
"

# Parse arguments
WHILE [[ $# -gt 0 ]]; do
    case "$1" in
        --with-vendoring) WITH_VENDORING=true; shift ;;
        --with-onboarding) WITH_ONBOARDING=true; shift ;;
        --with-flakes) WITH_FLAKES=true; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        --help) echo "$USAGE"; exit 0 ;;
        *) WORKSPACE_PATH="$1"; shift ;;
    esac
done

# Step 1: Build dependency graph with topological sorting
echo "📊 Building dependency graph..."
./target/debug/cargo-vendormod global-graph build "$WORKSPACE_PATH" --output-dir /tmp/graph

# Step 2: Process crates in topological order
echo "🔄 Processing crates with topological sorting..."
./target/debug/cargo-vendormod process-crates "$WORKSPACE_PATH" --output-dir /tmp/processed --layered-processing

# Step 3: Run selected workflows
if [[ "$WITH_VENDORING" == true ]]; then
    echo "📦 Running vendoring workflow..."
    ./scripts/minimal_workflow.sh "$WORKSPACE_PATH"
fi

if [[ "$WITH_ONBOARDING" == true ]]; then
    echo "🚀 Running onboarding workflow..."
    ./scripts/onboard.sh --workdir "$WORKSPACE_PATH"
fi

if [[ "$WITH_FLAKES" == true ]]; then
    echo "📋 Generating flake.nix files..."
    ./scripts/add_flake_nix.sh "$WORKSPACE_PATH"
fi

echo "✅ Complete workflow finished!"
```

#### 2. Integrate with LayerProcessor

**File:** `src/layer_processor.rs`

```rust
impl LayerProcessor {
    /// Run cargo-rail workflows after crate processing
    pub fn run_cargo_rail_workflows(&self, workspace_path: &Path) -> Result<()> {
        let workspace_str = workspace_path.to_string_lossy();
        
        // Vendoring workflow
        if self.config.run_vendoring {
            self.run_script("minimal_workflow.sh", &[&workspace_str])?;
        }
        
        // Onboarding workflow
        if self.config.run_onboarding {
            self.run_script("onboard.sh", &["--workdir", &workspace_str])?;
        }
        
        // Flake generation
        if self.config.generate_flakes {
            self.run_script("add_flake_nix.sh", &[&workspace_str])?;
        }
        
        Ok(())
    }
    
    /// Helper to run external scripts
    fn run_script(&self, script_name: &str, args: &[&str]) -> Result<()> {
        let script_path = self.scripts_dir.join(script_name);
        
        if !script_path.exists() {
            return Err(anyhow!("Script not found: {}", script_path.display()));
        }
        
        let mut cmd = Command::new(&script_path);
        cmd.args(args);
        
        if self.config.dry_run {
            println!("[DRY RUN] Would run: {:?}", cmd);
            return Ok(());
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Script failed: {}", error_msg));
        }
        
        Ok(())
    }
}
```

#### 3. Update CLI Arguments

**File:** `src/args.rs`

```rust
#[derive(Parser, Debug)]
pub struct ProcessCratesArgs {
    // ... existing args ...
    
    /// Run vendoring workflow after processing
    #[arg(long, default_value_t = false)]
    pub run_vendoring: bool,
    
    /// Run onboarding workflow after processing
    #[arg(long, default_value_t = false)]
    pub run_onboarding: bool,
    
    /// Generate flake.nix files after processing
    #[arg(long, default_value_t = false)]
    pub generate_flakes: bool,
}
```

#### 4. Update Main Processing

**File:** `src/main.rs`

```rust
fn cmd_process_crates(args: ProcessCratesArgs) -> Result<()> {
    // ... existing setup ...
    
    let mut processor = LayerProcessor::new(&workspace_path, &output_dir);
    
    // Process crates with topological sorting
    processor.process_all_crates()?;
    
    // Run cargo-rail workflows if requested
    if args.run_vendoring || args.run_onboarding || args.generate_flakes {
        println!("🚀 Running cargo-rail workflows...");
        processor.run_cargo_rail_workflows(&workspace_path)?;
    }
    
    println!("✅ Processing complete!");
    Ok(())
}
```

### Phase 3: Advanced Integration

**Goal:** Create unified workflow system

#### 1. Workflow Configuration System

**File:** `scripts/workflows/`

```bash
# Create workflow configuration files
mkdir -p scripts/workflows

# Example: full_onboarding.json
cat > scripts/workflows/full_onboarding.json << 'EOF'
{
  "name": "full_onboarding",
  "description": "Complete onboarding with topological sorting",
  "steps": [
    {
      "name": "dependency_graph",
      "command": "./target/debug/cargo-vendormod global-graph build {workspace} --output-dir /tmp/graph --include-dev --include-build"
    },
    {
      "name": "topological_processing",
      "command": "./target/debug/cargo-vendormod process-crates {workspace} --output-dir /tmp/processed --layered-processing"
    },
    {
      "name": "vendoring",
      "command": "./scripts/minimal_workflow.sh {workspace}",
      "optional": true
    },
    {
      "name": "onboarding",
      "command": "./scripts/onboard.sh --workdir {workspace}",
      "optional": true
    },
    {
      "name": "flake_generation",
      "command": "./scripts/add_flake_nix.sh {workspace}",
      "optional": true
    }
  ]
}
EOF
```

#### 2. Workflow Runner

**File:** `scripts/run_workflow.sh`

```bash
#!/bin/bash
# Unified workflow runner

set -e
set -u

WORKFLOW_NAME=${1:-full_onboarding}
WORKSPACE_PATH=${2:-.}
DRY_RUN=false

USAGE="Usage: $0 <workflow-name> <workspace-path> [--dry-run]

Example:
  $0 full_onboarding workload/workspaces/cargo-rail
  $0 full_onboarding workload/workspaces/cargo-rail --dry-run
"

while [[ $# -gt 2 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --help) echo "$USAGE"; exit 0 ;;
        *) echo "Unknown option: $1"; echo "$USAGE"; exit 1 ;;
    esac
done

WORKFLOW_CONFIG="scripts/workflows/${WORKFLOW_NAME}.json"

if [[ ! -f "$WORKFLOW_CONFIG" ]]; then
    echo "❌ Workflow not found: $WORKFLOW_CONFIG"
    echo "Available workflows:"
    ls scripts/workflows/*.json 2>/dev/null | sed 's|scripts/workflows/||;s|\.json||'
    exit 1
fi

echo "🚀 Starting workflow: $WORKFLOW_NAME"
echo "📁 Workspace: $WORKSPACE_PATH"
echo "📝 Dry run: $DRY_RUN"

# Read workflow configuration
STEPS=$(jq -r '.steps[] | @base64' "$WORKFLOW_CONFIG")

STEP_NUM=1
for STEP_B64 in $STEPS; do
    STEP=$(echo "$STEP_B64" | base64 --decode)
    
    NAME=$(echo "$STEP" | jq -r '.name')
    COMMAND=$(echo "$STEP" | jq -r '.command')
    OPTIONAL=$(echo "$STEP" | jq -r '.optional // "false"')
    
    # Replace workspace placeholder
    COMMAND=${COMMAND//\{workspace\}/$WORKSPACE_PATH}
    
    echo ""
    echo "📋 Step $STEP_NUM: $NAME"
    
    if [[ "$DRY_RUN" == true ]]; then
        echo "[DRY RUN] Would execute: $COMMAND"
    else
        if [[ "$OPTIONAL" == "true" ]]; then
            echo "🔧 Running optional step..."
        fi
        
        # Execute command
        if eval "$COMMAND"; then
            echo "✅ Step $STEP_NUM completed: $NAME"
        else
            echo "❌ Step $STEP_NUM failed: $NAME"
            if [[ "$OPTIONAL" == "true" ]]; then
                echo "ℹ️  Continuing (optional step)"
            else
                echo "💥 Workflow failed"
                exit 1
            fi
        fi
    fi
    
    STEP_NUM=$((STEP_NUM + 1))
done

echo ""
echo "🎉 Workflow completed: $WORKFLOW_NAME"
echo "📊 Executed $((STEP_NUM - 1)) steps"
```

#### 3. CLI Integration

**File:** `src/main.rs`

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...
    
    /// Run predefined workflows
    Workflow {
        /// Workflow name
        name: String,
        
        /// Workspace path
        workspace_path: PathBuf,
        
        /// Dry run mode
        #[arg(long)]
        dry_run: bool,
    },
}

async fn cmd_workflow(args: WorkflowArgs) -> Result<()> {
    let script_path = Path::new("scripts/run_workflow.sh");
    
    if !script_path.exists() {
        return Err(anyhow!("Workflow runner not found: {}", script_path.display()));
    }
    
    let mut cmd = Command::new(script_path);
    cmd.arg(&args.name)
       .arg(&args.workspace_path);
    
    if args.dry_run {
        cmd.arg("--dry-run");
    }
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Workflow failed: {}", error_msg));
    }
    
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
```

## 🎯 Integration Benefits

### Immediate Benefits

1. **Production-Ready Workflows** - Use cargo-rail's tested workflows
2. **Time Savings** - No need to recreate workflow logic
3. **Consistency** - Standardized workflows across projects
4. **Flexibility** - Multiple workflow options available

### Long-term Benefits

1. **Unified System** - Single interface for all workflows
2. **Extensibility** - Easy to add new workflows
3. **Maintainability** - Centralized workflow management
4. **Documentation** - Clear workflow definitions

## 📅 Implementation Timeline

| Phase | Duration | Focus |
|-------|----------|-------|
| 1. Immediate Integration | 1-2 days | Symlink scripts, basic testing |
| 2. Enhanced Integration | 3-5 days | Wrapper scripts, CLI integration |
| 3. Advanced Integration | 1 week | Workflow system, configuration |

### Phase 1: Immediate (Today)

- [ ] Create scripts directory
- [ ] Symlink cargo-rail scripts
- [ ] Test basic functionality
- [ ] Update quick start guide
- [ ] Document in progress tracker

### Phase 2: Enhanced (This Week)

- [ ] Create wrapper scripts
- [ ] Integrate with LayerProcessor
- [ ] Update CLI arguments
- [ ] Add workflow configuration
- [ ] Test end-to-end workflows

### Phase 3: Advanced (Next Week)

- [ ] Create workflow configuration system
- [ ] Implement workflow runner
- [ ] Add CLI workflow command
- [ ] Create comprehensive tests
- [ ] Update all documentation

## 🎯 Testing Plan

### Phase 1 Tests

```bash
# Test individual scripts
./scripts/minimal_workflow.sh workload/workspaces/cargo-rail --dry-run
./scripts/onboard.sh --workdir workload/workspaces/cargo-rail --dry-run
./scripts/add_flake_nix.sh workload/workspaces/cargo-rail --dry-run
```

### Phase 2 Tests

```bash
# Test wrapper script
./scripts/process_with_topological.sh workload/workspaces/cargo-rail --dry-run

# Test CLI integration
./target/debug/cargo-vendormod process-crates workload/workspaces/cargo-rail --with-vendoring --dry-run
```

### Phase 3 Tests

```bash
# Test workflow runner
./scripts/run_workflow.sh full_onboarding workload/workspaces/cargo-rail --dry-run

# Test CLI workflow command
./target/debug/cargo-vendormod workflow full_onboarding workload/workspaces/cargo-rail --dry-run
```

## 📎 Integration Checklist

### Setup

- [ ] Create `scripts/` directory
- [ ] Symlink cargo-rail scripts
- [ ] Verify scripts are executable
- [ ] Test basic script execution

### Documentation

- [ ] Update `QUICK_START_GUIDE.md`
- [ ] Update `PROJECT_PROGRESS_TRACKER.md`
- [ ] Create `SCRIPTS_INTEGRATION_PLAN.md` (this file)
- [ ] Add script documentation

### Code Integration

- [ ] Create wrapper scripts
- [ ] Update `LayerProcessor`
- [ ] Update CLI arguments
- [ ] Update main processing
- [ ] Create workflow system

### Testing

- [ ] Test individual scripts
- [ ] Test wrapper scripts
- [ ] Test CLI integration
- [ ] Test workflow runner
- [ ] Test end-to-end workflows

### Deployment

- [ ] Update all documentation
- [ ] Create user guides
- [ ] Announce new functionality
- [ ] Gather feedback
- [ ] Iterate based on feedback

## 🎯 Success Criteria

### Phase 1 Success
- [ ] All scripts accessible and executable
- [ ] Basic functionality verified
- [ ] Documentation updated
- [ ] No regressions in existing functionality

### Phase 2 Success
- [ ] Wrapper scripts working
- [ ] CLI integration functional
- [ ] LayerProcessor integration complete
- [ ] All tests passing

### Phase 3 Success
- [ ] Workflow system operational
- [ ] Configuration files created
- [ ] CLI workflow command working
- [ ] Comprehensive documentation
- [ ] User guides available

## 📋 Next Steps

### Immediate (Today)

```bash
# 1. Create scripts directory
mkdir -p scripts

# 2. Symlink cargo-rail scripts
ln -s ../workload/scripts/minimal_workflow.sh scripts/
ln -s ../workload/scripts/onboard.sh scripts/
ln -s ../workload/scripts/add_flake_nix.sh scripts/
ln -s ../workload/scripts/detect_fork_upstream.sh scripts/
ln -s ../workload/scripts/onboard_fork.sh scripts/

# 3. Test scripts
chmod +x scripts/*
./scripts/minimal_workflow.sh workload/workspaces/cargo-rail --dry-run

# 4. Update progress tracker
# Edit PROJECT_PROGRESS_TRACKER.md to reflect completion
```

### This Week

1. **Create wrapper scripts** - Integrate with topological sorting
2. **Update LayerProcessor** - Add workflow execution methods
3. **Test integration** - Verify end-to-end functionality
4. **Update documentation** - Reflect new capabilities

### Next Week

1. **Create workflow system** - Configuration-based workflows
2. **Implement workflow runner** - Unified execution system
3. **Add CLI command** - Native workflow support
4. **Comprehensive testing** - All scenarios covered

## 🎯 Summary

**Status:** Scripts discovered, integration planned
**Priority:** High - Provides production-ready workflows
**Next Step:** Create scripts directory and symlink scripts
**Blockers:** None - Ready to proceed

**Benefits:**
- ✅ Production-ready workflows from cargo-rail
- ✅ Time savings - no need to recreate logic
- ✅ Consistency across projects
- ✅ Flexible workflow options

**Let's integrate these scripts and leverage the existing functionality!** 🚀