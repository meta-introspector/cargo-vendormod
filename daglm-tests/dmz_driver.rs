//! DMZ Driver - Build all projects and capture 0xD8 0x2A tags
//!
//! This driver:
//! 1. Starts the eBPF monitor
//! 2. Rebuilds all DASL projects
//! 3. Runs tests
//! 4. Collects and records hits

use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== DMZ Driver: Building all projects with eBPF monitoring ===\n");

    // Step 1: Build eBPF monitor
    println!("[1/4] Building eBPF monitor...");
    build_ebpf()?;

    // Step 2: Start monitor in background (would need actual process spawning)
    println!("[2/4] Monitor ready (run d8_2a_user in separate terminal)");

    // Step 3: Build all projects
    let projects = discover_projects();
    println!("[3/4] Found {} projects to build\n", projects.len());

    for project in &projects {
        println!("Building: {}", project);
        build_project(project)?;
    }

    // Step 4: Run tests
    println!("\n[4/4] Running tests...");
    for project in &projects {
        println!("Testing: {}", project);
        test_project(project)?;
    }

    println!("\nDone! Collect hits from eBPF monitor output.");
    Ok(())
}

fn build_ebpf() -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args(["build", "--release", "--target", "bpfel-unknown-none"])
        .current_dir("d8_2a_ebpf")
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to build eBPF");
    }
    Ok(())
}

fn discover_projects() -> Vec<String> {
    let mut projects = Vec::new();
    let dasl_root = Path::new("/home/mdupont/dasl");

    // Rust workspace crates
    let rust_crates = [
        "rust/serde_ipld_dagcbor-escaped",
        "rust/ipld-core",
        "rust/n0_dasl",
        "rust/honggfuzz-rs",
        "rust/fractran_loader",
    ];
    for c in &rust_crates {
        projects.push(dasl_root.join(c).to_string_lossy().to_string());
    }

    // IMPL projects
    let impl_projects = [
        "IMPL/jacquard",
        "IMPL/microcosm-rs",
        "IMPL/atproto-crates",
    ];
    for p in &impl_projects {
        projects.push(dasl_root.join(p).to_string_lossy().to_string());
    }

    projects
}

fn build_project(path: &str) -> anyhow::Result<()> {
    let cargo_toml = Path::new(path).join("Cargo.toml");
    if cargo_toml.exists() {
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(path)
            .status()?;

        if !status.success() {
            eprintln!("Build failed for {}", path);
        }
    }
    Ok(())
}

fn test_project(path: &str) -> anyhow::Result<()> {
    let cargo_toml = Path::new(path).join("Cargo.toml");
    if cargo_toml.exists() {
        let status = Command::new("cargo")
            .args(["test", "--release"])
            .current_dir(path)
            .status()?;

        if !status.success() {
            eprintln!("Test failed for {}", path);
        }
    }
    Ok(())
}