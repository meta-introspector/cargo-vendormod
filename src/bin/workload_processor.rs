//! # Workload Processor Binary
//!
//! Recursively processes git submodules and Cargo.toml files with performance timing.

use anyhow::Result;
use cargo_vendormod::workload_processor::process_workload_recursive;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let root = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    println!("========================================");
    println!("  Cargo-Vendormod Workload Processor");
    println!("========================================\n");

    let start = std::time::Instant::now();
    let metrics = process_workload_recursive(&root)?;
    let _total_time = start.elapsed();

    println!("\n========================================");
    println!("  Performance Report");
    println!("========================================");
    println!("Total repositories: {}", metrics.total_repos);
    println!("Total Cargo.toml files: {}", metrics.total_cargoTOMs);
    println!("Processing time: {}ms", metrics.elapsed_ms);
    println!("Repos per second: {:.2}", metrics.repos_per_second);
    
    // Output JSON
    let json = serde_json::to_string_pretty(&metrics)?;
    println!("\nJSON Output:");
    println!("{}", json);
    
    Ok(())
}