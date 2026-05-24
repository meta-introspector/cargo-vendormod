//! # Workload Report Generator
//!
//! CLI that scans workload directories and produces performance reports.

use anyhow::Result;
use cargo_vendormod::workload_processor::process_workload_recursive;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    // Default to current directory if no args
    let root = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Workload Performance Report");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let metrics = process_workload_recursive(&root)?;

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Performance Metrics");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Total git repositories: {}", metrics.total_repos);
    println!("Total Cargo.toml files: {}", metrics.total_cargoTOMs);
    println!("Processing time: {}ms", metrics.elapsed_ms);
    println!("Repositories per second: {:.2}", metrics.repos_per_second);
    
    // Output JSON for further processing
    let json = serde_json::to_string_pretty(&metrics)?;
    eprintln!("\n--- JSON OUTPUT ---\n{}", json);
    
    Ok(())
}