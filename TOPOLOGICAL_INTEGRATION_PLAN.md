# Topological Sorting Integration Plan

## Overview
This document outlines how to integrate the existing topological sorting functionality from cargo-rail into our global dependency graph system.

## Current State Analysis

### Existing Topological Sorting in cargo-rail
**File**: `workload/workspaces/cargo-rail/src/graph/core.rs`

**Key Features**:
- ✅ Uses `petgraph::algo::toposort`
- ✅ Handles workspace-only subgraphs
- ✅ Filters out dev-dependencies to avoid cycles
- ✅ Returns crates in publish order (dependencies first)
- ✅ Comprehensive error handling for circular dependencies
- ✅ Well-tested with unit tests

**Key Method**: `publish_order()`
```rust
pub fn publish_order(&self) -> RailResult<Vec<String>> {
    // Build workspace-only subgraph
    // Filter out dev-dependencies
    // Run toposort
    // Return sorted list
}
```

### Our Global Dependency Graph
**File**: `src/global_dep_graph.rs`

**Current State**:
- ✅ Uses petgraph for dependency graph
- ✅ Has workspace member tracking
- ✅ Supports multiple dependency types
- ❌ No topological sorting implementation
- ❌ No publish order functionality

## Integration Strategy

### Option 1: Direct Integration (Recommended)
**Approach**: Add topological sorting directly to `GlobalDependencyGraphBuilder`

**Pros**:
- Simple and straightforward
- No external dependencies
- Consistent with existing architecture
- Easy to maintain

**Cons**:
- Some code duplication
- Need to adapt to our data structures

### Option 2: Reuse cargo-rail Code
**Approach**: Import and use cargo-rail's WorkspaceGraph

**Pros**:
- Leverage existing tested code
- Less duplication

**Cons**:
- Different data structures
- Complex integration
- Potential version conflicts

**Decision**: **Option 1 - Direct Integration**

## Implementation Plan

### Step 1: Add Topological Sorting to GlobalDependencyGraphBuilder

**File**: `src/global_dep_graph.rs`

```rust
impl GlobalDependencyGraphBuilder {
    /// Get workspace members in dependency order (dependencies first, dependents last).
    ///
    /// Returns crates in the order they should be processed: a crate's dependencies
    /// are always processed before the crate itself.
    ///
    /// Uses topological sort on the dependency graph to ensure correct ordering.
    ///
    /// # Errors
    /// Returns error if circular dependencies are detected among workspace members.
    pub fn get_publish_order(&self) -> Result<Vec<String>> {
        // Build a subgraph with only workspace members
        // This is critical: external dependencies can have cycles,
        // but workspace members should never have cycles (Cargo enforces this)
        let mut subgraph = Graph::<DependencyNode, DependencyEdgeType>::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        
        // Add only workspace member nodes
        for (node_id, &idx) in &self.node_map {
            if let Some(node) = self.graph.node_weight(idx) {
                if node.is_workspace_member {
                    let new_idx = subgraph.add_node(node.clone());
                    node_map.insert(node_id.clone(), new_idx);
                }
            }
        }
        
        // Add edges between workspace members only
        // Skip dev-dependencies as they don't affect processing order
        for (from_id, &from_idx) in &self.node_map {
            if let Some(from_node) = self.graph.node_weight(from_idx) {
                if from_node.is_workspace_member {
                    if let Some(&from_sub_idx) = node_map.get(from_id) {
                        // Check all outgoing edges from this workspace member
                        for edge in self.graph.edges_directed(from_idx, Direction::Outgoing) {
                            if let Some(to_id) = edge.target() {
                                if let Some(to_node) = self.graph.node_weight(to_id) {
                                    if to_node.is_workspace_member {
                                        if let Some(&to_sub_idx) = node_map.get(&self.graph[to_id].id) {
                                            // Skip dev-dependencies - they don't affect processing order
                                            if !edge.weight().is_dev_dependency {
                                                subgraph.add_edge(from_sub_idx, to_sub_idx, edge.weight().clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Run toposort on the workspace-only subgraph
        let sorted = toposort(&subgraph, None).map_err(|cycle| {
            anyhow::anyhow!(
                "Circular dependency detected involving workspace crate. This should not happen in a valid Cargo workspace."
            )
        })?;
        
        // Collect node IDs in dependency order
        let result: Vec<String> = sorted.into_iter().filter_map(|idx| {
            subgraph.node_weight(idx).map(|n| n.id.clone())
        }).collect();
        
        Ok(result)
    }
    
    /// Get topological order for all nodes (including external dependencies).
    ///
    /// This includes both workspace members and external crates in dependency order.
    /// Useful for layer 1 processing (external dependencies).
    ///
    /// # Note
    /// External dependencies may have cycles, so this method may fail.
    /// For workspace-only processing, use get_publish_order() instead.
    pub fn get_topological_order(&self) -> Result<Vec<String>> {
        // Run toposort on the full graph
        let sorted = toposort(&self.graph, None).map_err(|cycle| {
            anyhow::anyhow!("Circular dependency detected in dependency graph")
        })?;
        
        // Collect node IDs in topological order
        let result: Vec<String> = sorted.into_iter().filter_map(|idx| {
            self.graph.node_weight(idx).map(|n| n.id.clone())
        }).collect();
        
        Ok(result)
    }
}
```

### Step 2: Add Helper Methods

```rust
impl GlobalDependencyGraphBuilder {
    /// Get workspace members only (for layer 2 processing)
    pub fn get_workspace_members(&self) -> Vec<String> {
        self.graph.node_weights()
            .filter(|n| n.is_workspace_member)
            .map(|n| n.id.clone())
            .collect()
    }
    
    /// Get external dependencies only (for layer 1 processing)
    pub fn get_external_dependencies(&self) -> Vec<String> {
        self.graph.node_weights()
            .filter(|n| !n.is_workspace_member)
            .map(|n| n.id.clone())
            .collect()
    }
    
    /// Get topological order for external dependencies only
    pub fn get_external_dependency_order(&self) -> Result<Vec<String>> {
        // Build subgraph with only external dependencies
        let mut ext_subgraph = Graph::<DependencyNode, DependencyEdgeType>::new();
        let mut ext_node_map: HashMap<String, NodeIndex> = HashMap::new();
        
        // Add external dependency nodes
        for (node_id, &idx) in &self.node_map {
            if let Some(node) = self.graph.node_weight(idx) {
                if !node.is_workspace_member {
                    let new_idx = ext_subgraph.add_node(node.clone());
                    ext_node_map.insert(node_id.clone(), new_idx);
                }
            }
        }
        
        // Add edges between external dependencies
        for (from_id, &from_idx) in &self.node_map {
            if let Some(from_node) = self.graph.node_weight(from_idx) {
                if !from_node.is_workspace_member {
                    if let Some(&from_ext_idx) = ext_node_map.get(from_id) {
                        for edge in self.graph.edges_directed(from_idx, Direction::Outgoing) {
                            if let Some(to_id) = edge.target() {
                                if let Some(to_node) = self.graph.node_weight(to_id) {
                                    if !to_node.is_workspace_member {
                                        if let Some(&to_ext_idx) = ext_node_map.get(&self.graph[to_id].id) {
                                            ext_subgraph.add_edge(from_ext_idx, to_ext_idx, edge.weight().clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Run toposort on external dependency subgraph
        let sorted = toposort(&ext_subgraph, None).map_err(|cycle| {
            anyhow::anyhow!("Circular dependency detected among external dependencies")
        })?;
        
        Ok(sorted.into_iter().filter_map(|idx| {
            ext_subgraph.node_weight(idx).map(|n| n.id.clone())
        }).collect())
    }
}
```

### Step 3: Update GlobalDependencyGraph to Include Order Information

**File**: `src/global_dep_graph.rs`

```rust
/// Global dependency graph representing the entire Solana universe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub features: Vec<FeatureSet>,
    pub toml_structures: Vec<TomlStructure>,
    pub strongly_connected_components: Vec<Vec<String>>,
    pub partitions: Vec<GraphPartition>,
    pub metrics: GraphMetrics,
    pub workspace_path: PathBuf,
    
    // Add topological order information
    pub publish_order: Vec<String>,      // Workspace members in dependency order
    pub topological_order: Vec<String>,  // All nodes in topological order
    pub external_dependency_order: Vec<String>, // External deps in topological order
}
```

### Step 4: Update Build Method to Calculate Orders

```rust
impl GlobalDependencyGraphBuilder {
    /// Build the global dependency graph
    pub fn build_global_graph(&mut self) -> Result<GlobalDependencyGraph> {
        // ... existing code ...
        
        // Calculate topological orders
        let publish_order = self.get_publish_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate publish order: {}", e);
            Vec::new()
        });
        
        let topological_order = self.get_topological_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate topological order: {}", e);
            Vec::new()
        });
        
        let external_dependency_order = self.get_external_dependency_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate external dependency order: {}", e);
            Vec::new()
        });
        
        Ok(GlobalDependencyGraph {
            nodes,
            edges,
            features,
            toml_structures,
            strongly_connected_components: scc,
            partitions: Vec::new(),
            metrics,
            workspace_path: self.workspace_path.clone(),
            
            // Add order information
            publish_order,
            topological_order,
            external_dependency_order,
        })
    }
}
```

## Layered Processing Integration

### Layer Processor Implementation

**File**: `src/layer_processor.rs` (new)

```rust
use crate::global_dep_graph::{GlobalDependencyGraph, GlobalDependencyGraphBuilder};
use std::path::{Path, PathBuf};

pub struct LayerProcessor {
    graph: GlobalDependencyGraph,
    workspace_path: PathBuf,
    output_base: PathBuf,
    git_exe: PathBuf,
    home_dir: PathBuf,
}

impl LayerProcessor {
    pub fn new(
        graph: GlobalDependencyGraph,
        workspace_path: PathBuf,
        output_base: PathBuf,
        git_exe: PathBuf,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            graph,
            workspace_path,
            output_base,
            git_exe,
            home_dir,
        }
    }
    
    /// Process all crates in layered fashion
    pub fn process_layers(&self) -> Result<()> {
        println!("Processing Layer 1: External Dependencies");
        self.process_layer1()?;
        
        println!("Processing Layer 2: Workspace Members");
        self.process_layer2()?;
        
        Ok(())
    }
    
    /// Process Layer 1: External dependencies in topological order
    fn process_layer1(&self) -> Result<()> {
        for crate_id in &self.graph.external_dependency_order {
            self.process_crate(crate_id, true)?; // true = is_external
        }
        Ok(())
    }
    
    /// Process Layer 2: Workspace members in publish order
    fn process_layer2(&self) -> Result<()> {
        for crate_id in &self.graph.publish_order {
            self.process_crate(crate_id, false)?; // false = is_workspace
        }
        Ok(())
    }
    
    /// Process individual crate
    fn process_crate(&self, crate_id: &str, is_external: bool) -> Result<()> {
        println!("Processing crate: {}", crate_id);
        
        // Find crate info
        let crate_node = self.find_crate_node(crate_id)?;
        
        // Locate repository
        let repo_path = self.find_repository(&crate_node)?;
        
        // Checkout correct branch/version
        let branch = self.find_branch_for_version(&crate_node, &repo_path)?;
        self.checkout_branch(&repo_path, &branch)?;
        
        // Apply patches
        self.apply_patches(&repo_path)?;
        
        // Generate flake.nix
        self.generate_flake(&crate_node, &repo_path, is_external)?;
        
        // Compile standalone
        self.compile_standalone(&repo_path, is_external)?;
        
        Ok(())
    }
    
    // ... helper methods ...
}
```

## CLI Integration

### Add New Command

**File**: `src/args.rs`

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...
    
    /// Process crates with layered topological processing
    ProcessCrates {
        /// Path to workspace root directory
        workspace_path: PathBuf,
        
        /// Output directory for processed crates
        #[arg(long, default_value = "./processed")]
        output_dir: PathBuf,
        
        /// Generate Nix flakes for each crate
        #[arg(long, action = ArgAction::SetTrue)]
        generate_flakes: bool,
        
        /// Compile crates standalone
        #[arg(long, action = ArgAction::SetTrue)]
        compile_standalone: bool,
        
        /// Use layered processing (external deps first, then workspace)
        #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
        layered_processing: bool,
    },
}
```

### Implement Command Handler

**File**: `src/main.rs`

```rust
fn cmd_process_crates(
    workspace_path: PathBuf,
    output_dir: PathBuf,
    generate_flakes: bool,
    compile_standalone: bool,
    layered_processing: bool,
) -> Result<()> {
    println!("Starting layered crate processing...");
    
    // Build dependency graph
    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.clone());
    let graph = builder.build_global_graph()?;
    
    println!("Dependency Graph Metrics:");
    println!("- Total nodes: {}", graph.nodes.len());
    println!("- Total edges: {}", graph.edges.len());
    println!("- Workspace members: {}", graph.metrics.workspace_members);
    println!("- External dependencies: {}", 
        graph.nodes.len() - graph.metrics.workspace_members);
    
    println!("Processing Order:");
    println!("- External dependencies: {}", graph.external_dependency_order.len());
    println!("- Workspace members: {}", graph.publish_order.len());
    
    // Create layer processor
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/mdupont"));
    let processor = LayerProcessor::new(
        graph,
        workspace_path,
        output_dir,
        PathBuf::from("git"), // git executable
        home_dir,
    );
    
    // Process layers
    processor.process_layers()?;
    
    println!("Crate processing completed successfully!");
    Ok(())
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_publish_order_simple() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        
        // Create a simple test workspace
        // a -> b -> c (linear dependency chain)
        
        let mut builder = GlobalDependencyGraphBuilder::new(workspace_path);
        // Add test nodes and edges
        
        let order = builder.get_publish_order().unwrap();
        // Should return ["c", "b", "a"] (dependencies first)
        assert_eq!(order, vec!["c", "b", "a"]);
    }
    
    #[test]
    fn test_publish_order_complex() {
        // Test with more complex dependency graph
        // a -> b -> c
        //   -> d -> c
        // Should return ["c", "b", "d", "a"]
    }
    
    #[test]
    fn test_external_dependency_order() {
        // Test external dependency ordering
        // ext1 -> ext2
        // ext3 (no deps)
        // Should return ["ext2", "ext1", "ext3"] or ["ext3", "ext2", "ext1"]
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_layer_processing_simple() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let output_dir = temp_dir.path().join("output");
        
        // Create test workspace with simple dependencies
        // Setup test Cargo.toml files
        
        let result = cmd_process_crates(
            workspace_dir,
            output_dir,
            false, // no flakes
            false, // no compilation
            true,  // layered processing
        );
        
        assert!(result.is_ok());
        // Verify output structure
    }
}
```

## Error Handling

### Key Error Scenarios
1. **Circular Dependencies**: Handle gracefully with clear error messages
2. **Missing Repositories**: Provide helpful suggestions for resolution
3. **Git Operations Failure**: Retry logic and detailed error reporting
4. **Compilation Failures**: Capture and log build output

### Error Handling Implementation
```rust
impl LayerProcessor {
    fn process_crate(&self, crate_id: &str, is_external: bool) -> Result<()> {
        // Wrap each operation with detailed error handling
        match self.find_crate_node(crate_id) {
            Ok(node) => {
                match self.find_repository(&node) {
                    Ok(repo_path) => {
                        // Continue processing
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to find repository for {}: {}. Searched in: \n{}",
                            crate_id, e,
                            self.list_search_locations(&node)
                        ));
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Crate {} not found in dependency graph: {}",
                    crate_id, e
                ));
            }
        }
        
        // Continue with other operations...
    }
    
    fn list_search_locations(&self, node: &DependencyNode) -> String {
        let mut locations = Vec::new();
        
        // List where we searched
        locations.push(format!("~/git/{}/*.git", node.source));
        locations.push(format!("submodules/{}", node.crate_name));
        
        locations.join("\n")
    }
}
```

## Performance Considerations

### Optimization Opportunities
1. **Parallel Processing**: Process independent crates in parallel
2. **Caching**: Cache repository locations and git operations
3. **Incremental Processing**: Skip already processed crates
4. **Memory Efficiency**: Stream large outputs instead of buffering

### Parallel Processing Implementation
```rust
use rayon::prelude::*;

impl LayerProcessor {
    fn process_layer1(&self) -> Result<()> {
        // Process external dependencies in parallel where possible
        self.graph.external_dependency_order
            .par_iter()
            .try_for_each(|crate_id| {
                self.process_crate(crate_id, true)
            })
    }
}
```

## Documentation Updates

### User Guide Updates
**File**: `USER_GUIDE.md`

```markdown
## Layered Crate Processing

The `process-crates` command processes crates in a layered fashion:

### Layer 1: External Dependencies
- Processes external crates in topological order
- Ensures dependencies are built before dependents
- Generates standalone flakes for each external crate

### Layer 2: Workspace Members
- Processes workspace crates in publish order
- Uses Layer 1 outputs as inputs
- Generates root flake for the workspace

### Usage

```bash
# Basic processing
cargo run -- process-crates --workspace-path /path/to/workspace

# With flake generation
cargo run -- process-crates --workspace-path /path/to/workspace --generate-flakes

# With compilation
cargo run -- process-crates --workspace-path /path/to/workspace --compile-standalone
```

### Output Structure

```
output/
├── layer1/
│   ├── crate1/
│   │   ├── flake.nix
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── crate2/
│   │   ├── flake.nix
│   │   └── ...
│   └── ...
├── layer2/
│   ├── workspace-flake.nix
│   ├── Cargo.toml
│   └── crates/
└── summary.json
```
```

## Implementation Checklist

### Phase 1: Core Topological Sorting
- [ ] Add `get_publish_order()` to `GlobalDependencyGraphBuilder`
- [ ] Add `get_topological_order()` to `GlobalDependencyGraphBuilder`
- [ ] Add `get_external_dependency_order()` to `GlobalDependencyGraphBuilder`
- [ ] Update `GlobalDependencyGraph` struct to include order information
- [ ] Update `build_global_graph()` to calculate orders
- [ ] Write unit tests for sorting algorithms

### Phase 2: Layer Processor
- [ ] Create `LayerProcessor` struct
- [ ] Implement `process_layers()` method
- [ ] Implement `process_layer1()` for external dependencies
- [ ] Implement `process_layer2()` for workspace members
- [ ] Implement individual crate processing
- [ ] Write integration tests

### Phase 3: CLI Integration
- [ ] Add `ProcessCrates` command to CLI
- [ ] Implement command handler
- [ ] Add help text and documentation
- [ ] Test CLI integration

### Phase 4: Enhancements
- [ ] Add parallel processing
- [ ] Add caching for git operations
- [ ] Add incremental processing
- [ ] Add progress reporting
- [ ] Add detailed logging

## Risk Assessment

### High Risk
- **Topological Sorting**: Complex dependency graphs may have edge cases
- **Git Operations**: Network operations can fail unpredictably
- **Performance**: Large workspaces may be slow without optimization

### Mitigation Strategies
1. **Comprehensive Testing**: Test with various dependency graph patterns
2. **Robust Error Handling**: Provide clear error messages and recovery options
3. **Performance Profiling**: Identify and optimize bottlenecks
4. **Incremental Rollout**: Test with small workspaces first

## Conclusion

This integration plan provides a clear path to add topological sorting and layered processing to our crate processing pipeline. By leveraging the existing cargo-rail implementation as a reference and adapting it to our global dependency graph, we can efficiently implement the requested functionality while maintaining code quality and testability.

The layered approach ensures that dependencies are always processed before dependents, enabling reliable standalone compilation and flake generation. This architecture will support the full crate processing pipeline as specified in the requirements.
