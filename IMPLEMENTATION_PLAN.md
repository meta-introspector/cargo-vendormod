# Crate Processing Pipeline Implementation Plan

## Overview
This document outlines the implementation plan for the comprehensive crate processing pipeline as requested. The pipeline will process each crate, create dependency trees, manage git repositories, apply patches, and compile standalone flakes.

## Current State Analysis

### Existing Functionality
1. **Dependency Analysis**: ✅ `src/global_dep_graph.rs` has comprehensive graph analysis
2. **Git Operations**: ✅ `src/vendoring.rs` and `src/workload.rs` handle git operations
3. **Submodule Management**: ✅ Basic submodule add/checkout functionality
4. **Patch Management**: ✅ Cargo.toml patch generation
5. **Graph Analysis**: ✅ Petgraph-based dependency graph with SCC detection

### Missing Functionality
1. **Topological Sorting**: ❌ No explicit topological sort for compilation order
2. **Flake.nix Generation**: ❌ No Nix flake generation
3. **Layered Processing**: ❌ No layer 1/2 concept implemented
4. **Standalone Compilation**: ❌ No standalone crate compilation
5. **Branch/Version Mapping**: ❌ Basic version resolution but no sophisticated mapping

## Implementation Roadmap

### Phase 1: Dependency Tree Analysis (Current)
**Files**: `src/global_dep_graph.rs`, `src/workload.rs`

#### Current Implementation
- ✅ Dependency graph building using cargo-metadata
- ✅ Graph visualization with DOT format
- ✅ Strongly connected component detection
- ✅ Basic git repository handling

#### Required Enhancements
1. **Add Topological Sorting**:
   ```rust
   // In src/global_dep_graph.rs
   pub fn get_topological_order(&self) -> Result<Vec<String>> {
       use petgraph::algo::toposort;
       
       let node_indices: Vec<NodeIndex> = self.graph.node_indices().collect();
       match toposort(&self.graph, None) {
           Ok(order) => Ok(order.iter().filter_map(|&idx| {
               self.graph.node_weight(idx).map(|n| n.id.clone())
           }).collect()),
           Err(_) => Err(anyhow::anyhow!("Circular dependencies prevent topological sorting"))
       }
   }
   ```

### Phase 2: Git Repository Management
**Files**: `src/repo_collection.rs`, `src/vendoring.rs`

#### Current Implementation
- ✅ Bare repository cloning
- ✅ Version branch creation
- ✅ Submodule addition
- ✅ Basic commit resolution

#### Required Enhancements
1. **Enhanced Repository Location Logic**:
   ```rust
   fn find_repository_location(
       repo_url: &str,
       home_dir: &Path,
       submodules_dir: &Path
   ) -> Option<PathBuf> {
       // Parse github.com/owner/repo.git -> owner, repo
       let (owner, repo_name) = parse_github_url(repo_url)?;
       
       // Check 1: ~/git/hostname/org/repo.git (bare)
       let home_bare = home_dir.join("git").join("github.com").join(owner).join(format!("{}.git", repo_name));
       if home_bare.exists() {
           return Some(home_bare);
       }
       
       // Check 2: submodules/repo (worktree)
       let submodule_path = submodules_dir.join(&repo_name);
       if submodule_path.exists() {
           return Some(submodule_path);
       }
       
       None
   }
   ```

2. **Branch/Version Mapping**:
   ```rust
   fn find_branch_for_version(
       repo_path: &Path,
       version: &str,
       git_exe: &Path
   ) -> Result<Option<String>> {
       // Try exact branch match
       let exact_branch = check_branch_exists(repo_path, version, git_exe)?;
       if exact_branch {
           return Ok(Some(version.to_string()));
       }
       
       // Try version tags
       let tags = get_git_tags(repo_path, git_exe)?;
       for tag in tags {
           if tag.starts_with(version) || tag.contains(version) {
               return Ok(Some(tag));
           }
       }
       
       // Try semantic version patterns
       let semver_pattern = format!("v{}", version);
       if check_branch_exists(repo_path, &semver_pattern, git_exe)? {
           return Ok(Some(semver_pattern));
       }
       
       Ok(None)
   }
   ```

### Phase 3: Patch Management
**Files**: `src/patch.rs`, `src/workload.rs`

#### Current Implementation
- ✅ Basic patch generation for Cargo.toml
- ✅ Submodule patch configuration

#### Required Enhancements
1. **Comprehensive Patch Application**:
   ```rust
   fn apply_patches_to_repository(
       repo_path: &Path,
       patches_dir: &Path,
       git_exe: &Path
   ) -> Result<()> {
       // Find all patch files for this repository
       let patch_files = find_patch_files(patches_dir, repo_path)?;
       
       // Apply each patch
       for patch_file in patch_files {
           let output = Command::new(git_exe)
               .arg("-C")
               .arg(repo_path)
               .arg("apply")
               .arg("--index")
               .arg(&patch_file)
               .output()?;
           
           if !output.status.success() {
               return Err(anyhow::anyhow!(
                   "Failed to apply patch {}: {}",
                   patch_file.display(),
                   String::from_utf8_lossy(&output.stderr)
               ));
           }
       }
       
       Ok(())
   }
   ```

### Phase 4: Nix Flake Generation
**Files**: `src/nix_flake.rs` (new file needed)

#### New Implementation Required
1. **Flake.nix Template Generation**:
   ```rust
   pub struct NixFlakeGenerator {
       workspace_path: PathBuf,
       output_dir: PathBuf,
       crate_name: String,
       version: String,
       dependencies: Vec<String>,
   }
   
   impl NixFlakeGenerator {
       pub fn new(
           workspace_path: PathBuf,
           output_dir: PathBuf,
           crate_name: String,
           version: String,
           dependencies: Vec<String>
       ) -> Self {
           Self {
               workspace_path,
               output_dir,
               crate_name,
               version,
               dependencies,
           }
       }
       
       pub fn generate_flake(&self) -> Result<()> {
           let flake_content = format!(r#"
{{
  description = "{} v{} - Auto-generated flake";
  
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    {} 
  }};
  
  outputs = {{ self, nixpkgs, flake-utils, ... }}@inputs: {{
    packages = {{
      default = self.packages.${{system}}.{}; 
    }};
    
    packages.${{system}} = {{
      {} = inputs.nixpkgs.legacyPackages.${{system}}.callPackage ./. {{ }};
    }};
  }};
}}
"#, 
           self.crate_name, self.version,
           self.generate_input_declarations(),
           self.crate_name,
           self.crate_name
           );
           
           let flake_path = self.output_dir.join("flake.nix");
           fs::write(flake_path, flake_content)?;
           
           Ok(())
       }
       
       fn generate_input_declarations(&self) -> String {
           self.dependencies.iter().map(|dep| {
               format!("    {}.url = \"path:../{}\";", dep, dep)
           }).collect::<Vec<_>>().join("\n")
       }
   }
   ```

### Phase 5: Layered Processing System
**Files**: `src/layer_processor.rs` (new file needed)

#### New Implementation Required
1. **Layer Processing Engine**:
   ```rust
   pub struct LayerProcessor {
       graph: GlobalDependencyGraph,
       workspace_path: PathBuf,
       output_base: PathBuf,
       git_exe: PathBuf,
   }
   
   impl LayerProcessor {
       pub fn new(
           graph: GlobalDependencyGraph,
           workspace_path: PathBuf,
           output_base: PathBuf,
           git_exe: PathBuf
       ) -> Self {
           Self {
               graph,
               workspace_path,
               output_base,
               git_exe,
           }
       }
       
       pub fn process_layers(&self) -> Result<()> {
           // Get topological order for layer 1 (dependencies)
           let layer1_order = self.get_layer1_order()?;
           
           // Process layer 1 crates
           self.process_layer1_crates(&layer1_order)?;
           
           // Process layer 2 (main workspace)
           self.process_layer2_workspace()?;
           
           Ok(())
       }
       
       fn get_layer1_order(&self) -> Result<Vec<String>> {
           // Get all non-workspace nodes (dependencies)
           let dep_nodes: Vec<_> = self.graph.nodes.iter()
               .filter(|n| !n.is_workspace_member)
               .map(|n| n.id.clone())
               .collect();
           
           // Create subgraph of just dependencies
           // Perform topological sort on this subgraph
           // Return sorted list
           
           Ok(dep_nodes)
       }
       
       fn process_layer1_crates(&self, crate_order: &[String]) -> Result<()> {
           for crate_id in crate_order {
               // Find crate info
               // Locate repository
               // Checkout correct branch
               // Apply patches
               // Generate flake.nix
               // Compile standalone
               // Push to local cache
           }
           Ok(())
       }
       
       fn process_layer2_workspace(&self) -> Result<()> {
           // Process main workspace with layer 1 as input
           // Generate root flake.nix
           // Compile full workspace
           Ok(())
       }
   }
   ```

### Phase 6: Standalone Compilation
**Files**: `src/compilation.rs` (new file needed)

#### New Implementation Required
1. **Standalone Crate Compiler**:
   ```rust
   pub struct StandaloneCompiler {
       crate_path: PathBuf,
       output_dir: PathBuf,
       nix_exe: PathBuf,
       git_exe: PathBuf,
   }
   
   impl StandaloneCompiler {
       pub fn compile_standalone(&self) -> Result<()> {
           // Change to crate directory
           // Run cargo build
           // Generate nix derivation
           // Build with nix
           // Cache result
           
           let output = Command::new("cargo")
               .current_dir(&self.crate_path)
               .arg("build")
               .arg("--release")
               .output()?;
           
           if !output.status.success() {
               return Err(anyhow::anyhow!(
                   "Failed to build {}: {}",
                   self.crate_path.display(),
                   String::from_utf8_lossy(&output.stderr)
               ));
           }
           
           // Generate nix derivation
           self.generate_nix_derivation()?;
           
           // Build with nix
           self.build_with_nix()?;
           
           Ok(())
       }
       
       fn generate_nix_derivation(&self) -> Result<()> {
           // Use cargo2nix or custom template
           // Generate default.nix
           Ok(())
       }
       
       fn build_with_nix(&self) -> Result<()> {
           let output = Command::new("nix")
               .arg("build")
               .arg("-f")
               .arg("default.nix")
               .current_dir(&self.crate_path)
               .output()?;
           
           if !output.status.success() {
               return Err(anyhow::anyhow!(
                   "Nix build failed: {}",
                   String::from_utf8_lossy(&output.stderr)
               ));
           }
           
           Ok(())
       }
   }
   ```

## Integration Plan

### Main Workflow Integration
**File**: `src/main.rs`

Add new command:
```rust
#[derive(Subcommand, Debug)]
enum Commands {
    // ... existing commands ...
    /// Process crates with full pipeline: tree analysis, git management, patching, flake generation, compilation
    ProcessCrates {
        /// Path to workspace root directory
        workspace_path: PathBuf,
        /// Output directory for processed crates
        #[arg(long, default_value = "./processed")]
        output_dir: PathBuf,
        /// Generate Nix flakes
        #[arg(long, action = ArgAction::SetTrue)]
        generate_flakes: bool,
        /// Compile standalone crates
        #[arg(long, action = ArgAction::SetTrue)]
        compile_standalone: bool,
        /// Process in layers (1 = deps, 2 = workspace)
        #[arg(long, action = ArgAction::SetTrue)]
        layered_processing: bool,
    },
}
```

### Command Implementation
```rust
fn cmd_process_crates(
    workspace_path: PathBuf,
    output_dir: PathBuf,
    generate_flakes: bool,
    compile_standalone: bool,
    layered_processing: bool,
) -> Result<()> {
    println!("Starting comprehensive crate processing...");
    
    // Step 1: Build dependency graph
    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.clone());
    let graph = builder.build_global_graph()?;
    
    // Step 2: Get topological order
    let topo_order = builder.get_topological_order()?;
    
    // Step 3: Process each crate
    let processor = CrateProcessor::new(graph, workspace_path, output_dir, layered_processing);
    processor.process_all_crates(generate_flakes, compile_standalone)?;
    
    println!("Crate processing completed successfully!");
    Ok(())
}
```

## Implementation Priority

### High Priority (Must Have)
1. **Topological Sorting**: Critical for compilation order
2. **Enhanced Git Repository Management**: Core functionality
3. **Layered Processing**: Fundamental to the architecture
4. **Basic Flake Generation**: Required for Nix integration

### Medium Priority (Should Have)
1. **Advanced Patch Management**: Important but can be basic initially
2. **Standalone Compilation**: Useful but can use cargo directly
3. **Branch/Version Mapping**: Can start with simple logic
4. **Error Handling**: Robust but can be improved later

### Low Priority (Nice to Have)
1. **Performance Optimization**: Can optimize after basic functionality works
2. **Advanced Nix Features**: Start with basic flakes
3. **Parallel Processing**: Can add later for speed
4. **Comprehensive Testing**: Start with basic test coverage

## Testing Strategy

### Unit Tests
- Topological sorting algorithm
- Git repository location logic
- Branch/version mapping
- Flake generation templates

### Integration Tests
- End-to-end crate processing
- Layered processing workflow
- Patch application and compilation
- Nix flake generation and usage

### Test Data
- Sample Rust workspace with multiple crates
- Various dependency scenarios (simple, circular, complex)
- Different git repository configurations

## Risk Assessment

### High Risk
- **Topological Sorting**: Complex dependency graphs may have issues
- **Git Operations**: Network operations can fail
- **Nix Integration**: Requires Nix environment

### Medium Risk
- **Layered Processing**: Logic complexity
- **Patch Management**: Conflict resolution
- **Performance**: Large workspaces may be slow

### Low Risk
- **Flake Generation**: Template-based, easy to fix
- **Compilation**: Uses standard tools
- **Error Handling**: Can be improved incrementally

## Timeline Estimate

### Phase 1: Core Functionality (2-3 days)
- Topological sorting implementation
- Basic git repository management
- Layered processing framework
- Simple flake generation

### Phase 2: Integration (1-2 days)
- Connect all components
- Basic error handling
- Command-line interface
- Initial testing

### Phase 3: Enhancement (3-5 days)
- Advanced patch management
- Standalone compilation
- Performance optimization
- Comprehensive testing
- Documentation

## Next Steps

1. **Implement Topological Sorting**: Add to `GlobalDependencyGraphBuilder`
2. **Enhance Git Management**: Improve repository location and branch logic
3. **Create Layer Processor**: Implement layered processing system
4. **Add Flake Generation**: Basic Nix flake template generation
5. **Integrate Components**: Connect all parts in main workflow
6. **Test and Debug**: Verify functionality with sample workspaces

## Implementation Checklist

- [ ] Add topological sorting to `GlobalDependencyGraphBuilder`
- [ ] Enhance git repository location logic
- [ ] Implement branch/version mapping
- [ ] Create patch management system
- [ ] Develop Nix flake generator
- [ ] Build layered processing engine
- [ ] Implement standalone compiler
- [ ] Add new CLI command
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update documentation
- [ ] Performance testing
- [ ] User acceptance testing

## Conclusion

This implementation plan provides a comprehensive roadmap for building the requested crate processing pipeline. The approach leverages existing functionality while adding the missing components in a logical sequence. The layered architecture and topological sorting will enable efficient processing of complex dependency graphs, while the Nix flake integration provides the foundation for reproducible builds.
