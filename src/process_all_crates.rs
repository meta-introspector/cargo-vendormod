use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader};
use rayon::prelude::*;
use log::{info, error};

use crate::layer_processor::LayerProcessor;
use crate::global_dep_graph::GlobalDependencyGraphBuilder;

/// Process all crates from a list of Cargo.toml files
pub fn process_all_crates_from_file(
    input_file: &Path,
    output_base: &Path,
    max_parallel: usize,
) -> Result<()> {
    info!("Starting comprehensive crate processing");
    info!("Input file: {}", input_file.display());
    info!("Output base: {}", output_base.display());
    info!("Max parallel: {}", max_parallel);
    
    // Read all Cargo.toml paths
    let file = fs::File::open(input_file)
        .context("Failed to open input file")?;
    let reader = BufReader::new(file);
    
    let cargo_toml_paths: Vec<PathBuf> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect();
    
    info!("Found {} Cargo.toml files to process", cargo_toml_paths.len());
    
    // Create output directory
    fs::create_dir_all(output_base)
        .context("Failed to create output directory")?;
    
    // Process crates in parallel
    let results: Vec<(PathBuf, Result<(PathBuf, bool)>)> = cargo_toml_paths.clone()
        .into_par_iter()
        .map(|cargo_toml_path| {
            let result = process_single_crate(&cargo_toml_path, output_base);
            (cargo_toml_path.clone(), result)
        })
        .collect();
    
    // Analyze results
    let mut success_count = 0;
    let mut failed_count = 0;
    
    for (cargo_toml_path, result) in results {
        match result {
            Ok((crate_path, _)) => {
                success_count += 1;
                info!("✅ Successfully processed: {}", crate_path.display());
            }
            Err(e) => {
                failed_count += 1;
                error!("❌ Failed to process: {}: {}", 
                    cargo_toml_path.display(), e);
            }
        }
    }
    
    info!("🏁 Processing complete!");
    info!("✅ Successfully processed: {}", success_count);
    info!("❌ Failed to process: {}", failed_count);
    info!("📈 Success rate: {}%", 
        if cargo_toml_paths.len() > 0 {
            (success_count * 100) / cargo_toml_paths.len()
        } else {
            0
        }
    );
    
    if failed_count > 0 {
        Err(anyhow::anyhow!("Some crates failed to process"))
    } else {
        Ok(())
    }
}

/// Process a single crate from its Cargo.toml path
fn process_single_crate(
    cargo_toml_path: &Path,
    output_base: &Path,
) -> Result<(PathBuf, bool)> {
    let crate_dir = cargo_toml_path.parent()
        .context("Failed to get parent directory")?;
    let crate_name = crate_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    info!("📦 Processing crate: {} ({})", crate_name, cargo_toml_path.display());
    
    // Create workspace directory structure
    let workspace_dir = output_base.join("workspaces").join(&crate_name);
    fs::create_dir_all(&workspace_dir)
        .context("Failed to create workspace directory")?;
    
    // Copy Cargo.toml to workspace
    let dest_cargo_toml = workspace_dir.join("Cargo.toml");
    fs::copy(cargo_toml_path, &dest_cargo_toml)
        .context("Failed to copy Cargo.toml")?;
    
    // Create minimal src directory if it doesn't exist
    let src_dir = workspace_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)
            .context("Failed to create src directory")?;
        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(lib_rs_path, format!("// Minimal implementation for {}\
", crate_name))
            .context("Failed to write lib.rs")?;
    }
    
    // Create output directory for this crate
    let crate_output_dir = output_base.join(format!("{}_output", crate_name));
    
    // Process both layers
    process_crate_layers(&workspace_dir, &crate_output_dir)?;
    
    Ok((cargo_toml_path.to_path_buf(), true))
}

/// Process crate layers (external dependencies + workspace members)
fn process_crate_layers(
    workspace_path: &Path,
    output_dir: &Path,
) -> Result<()> {
    info!("Building dependency graph for {}", workspace_path.display());
    
    // Build dependency graph
    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.to_path_buf());
    let graph = builder.build_global_graph()
        .context("Failed to build dependency graph")?;
    
    info!("Dependency Graph Metrics:");
    info!("- Total nodes: {}", graph.nodes.len());
    info!("- Workspace members: {}", graph.metrics.workspace_members);
    info!("- External dependencies: {}", 
        graph.nodes.len() - graph.metrics.workspace_members);
    
    // Create output directory
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;
    
    // Create layer processor
    let home_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/mdupont"));
    
    let processor = LayerProcessor::new(
        graph,
        workspace_path.to_path_buf(),
        output_dir.to_path_buf(),
        PathBuf::from("git"),
        home_dir,
    );
    
    // Process all layers
    processor.process_layers()
        .context("Failed to process layers")?;
    
    Ok(())
}

/// Add command to args.rs
pub fn add_process_all_command() {
    // This will be integrated into the main CLI
}