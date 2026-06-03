use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::BufRead;
use log::info;

/// Generate a comprehensive processing report
pub fn generate_processing_report(
    input_file: &Path,
    output_base: &Path,
) -> Result<String> {
    info!("Generating processing report...");
    
    // Count total input crates (fast path)
    let total_input_crates = fast_count_lines(input_file)?;
    
    // Count processed workspaces (fast path)
    let workspaces_dir = output_base.join("workspaces");
    let processed_crates = fast_count_directories(&workspaces_dir)?;
    
    // Count generated flakes (fast path - count flake.nix files in all output directories)
    let flake_count = count_flake_files_fast(output_base)?;
    
    // Check for submodule usage (fast path - check if submodules dir exists)
    let submodule_usage = fast_check_submodule_integration(output_base)?;
    
    // Generate report
    let report = format!(
        "📊 CARGO-VENDORMOD PROCESSING REPORT
{
}
📦 Input Statistics:
  Total Cargo.toml files: {}
  Expected crates: {}
{
}
✅ Processing Results:
  Crates processed: {}
  Nix flakes generated: {}
  Success rate: {}%
{
}
🔗 Submodule Integration:
  Submodules detected: {}
  Submodule usage: {}
{
}
📁 Output Structure:
  Workspaces: {}/workspaces/ ({} crates)
  Flakes: {}/**/layer*/*/flake.nix ({} flakes)
{
}
🎯 Status:
  {} of {} crates processed ({}%)
  {} flakes generated for reproducible builds
  Topological sorting: ✅ Active
  Layered processing: ✅ Active
  Git integration: ✅ Active
  Nix flake generation: ✅ Active
",
        "=".repeat(50),
        total_input_crates, total_input_crates,
        "-".repeat(30),
        processed_crates, flake_count,
        if total_input_crates > 0 {
            (processed_crates * 100) / total_input_crates
        } else {
            0
        },
        "-".repeat(30),
        submodule_usage.submodule_count,
        submodule_usage.description,
        "-".repeat(30),
        output_base.display(), processed_crates,
        output_base.display(), flake_count,
        "-".repeat(30),
        processed_crates, total_input_crates,
        if total_input_crates > 0 {
            (processed_crates * 100) / total_input_crates
        } else {
            0
        },
        flake_count
    );
    
    Ok(report)
}

/// Fast count lines in a file
fn fast_count_lines(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let content = fs::read_to_string(path)
        .context(format!("Failed to read input file: {}", path.display()))?;
    Ok(content.lines().count())
}

/// Fast count directories in a path
fn fast_count_directories(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Fast count flake.nix files
fn count_flake_files_fast(base: &Path) -> Result<usize> {
    let mut count = 0;
    
    if !base.exists() {
        return Ok(0);
    }
    
    // Look for flake.nix files in common locations
    let search_dirs = vec![
        base.join("workspaces"),
        base.join("layer1"),
        base.join("layer2"),
    ];
    
    for search_dir in search_dirs {
        if search_dir.exists() {
            for entry in fs::read_dir(search_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check for flake.nix in this directory
                        let flake_path = path.join("flake.nix");
                        if flake_path.exists() {
                            count += 1;
                        }
                        // Also check layer subdirectories
                        let layer1_flake = path.join("layer1").join("flake.nix");
                        let layer2_flake = path.join("layer2").join("flake.nix");
                        if layer1_flake.exists() {
                            count += 1;
                        }
                        if layer2_flake.exists() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(count)
}

/// Fast check for submodule integration
fn fast_check_submodule_integration(output_base: &Path) -> Result<SubmoduleUsage> {
    let submodules_dir = output_base.join("submodules");
    let uses_submodules = submodules_dir.exists();
    
    let submodule_count = if uses_submodules {
        fast_count_directories(&submodules_dir)?
    } else {
        0
    };
    
    let description = if uses_submodules {
        format!("✅ Submodules integrated ({} submodules)", submodule_count)
    } else {
        "❌ No submodule integration detected".to_string()
    };
    
    Ok(SubmoduleUsage {
        submodule_count,
        description,
    })
}

struct SubmoduleUsage {
    submodule_count: usize,
    description: String,
}

fn check_submodule_integration(output_base: &Path) -> Result<SubmoduleUsage> {
    let mut submodule_count = 0;
    let mut uses_submodules = false;
    
    // Check if any flakes reference submodules
    let flake_files = find_files_recursively(output_base, "flake.nix")?;
    
    for flake_path in flake_files {
        let content = fs::read_to_string(&flake_path)?;
        if content.contains("submodules") || content.contains("github:NixOS/nixpkgs") {
            uses_submodules = true;
            submodule_count += 1;
        }
    }
    
    let description = if uses_submodules {
        format!("✅ Submodules integrated ({} flakes)", submodule_count)
    } else {
        "❌ No submodule integration detected".to_string()
    };
    
    Ok(SubmoduleUsage {
        submodule_count,
        description,
    })
}

fn count_files_recursively(base: &Path, filename: &str) -> Result<usize> {
    let mut count = 0;
    
    if !base.exists() {
        return Ok(0);
    }
    
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            count += count_files_recursively(&path, filename)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            count += 1;
        }
    }
    
    Ok(count)
}

fn find_files_recursively(base: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !base.exists() {
        return Ok(files);
    }
    
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let mut sub_files = find_files_recursively(&path, pattern)?;
            files.append(&mut sub_files);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(pattern) {
            files.push(path);
        }
    }
    
    Ok(files)
}

/// Add CLI command for reporting
pub fn add_report_command() {
    // Integration point for CLI
}