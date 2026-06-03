use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for workspace creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub base_dir: PathBuf,
    pub submodules_dir: PathBuf,
    pub edition: String,
    pub crates: Vec<CrateConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateConfig {
    pub display_name: String,
    pub actual_name: String,
    pub source_type: String, // "submodule", "crates.io", "local"
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/workload/workspaces/rust_toolchain"),
            submodules_dir: PathBuf::from("/mnt/data1/nix/vendor/rust/cargo2nix/submodules"),
            edition: "2021".to_string(),
            crates: vec![
                CrateConfig {
                    display_name: "cargo".to_string(),
                    actual_name: "cargo".to_string(),
                    source_type: "submodule".to_string(),
                },
                CrateConfig {
                    display_name: "cargo_metadata".to_string(),
                    actual_name: "cargo_metadata".to_string(),
                    source_type: "submodule".to_string(),
                },
                CrateConfig {
                    display_name: "compiler-builtins".to_string(),
                    actual_name: "compiler-builtins".to_string(),
                    source_type: "submodule".to_string(),
                },
                CrateConfig {
                    display_name: "rustc-demangle".to_string(),
                    actual_name: "rustc-demangle".to_string(),
                    source_type: "submodule".to_string(),
                },
                CrateConfig {
                    display_name: "rustc_apfloat".to_string(),
                    actual_name: "rustc_apfloat".to_string(),
                    source_type: "submodule".to_string(),
                },
            ],
        }
    }
}

pub fn fix_rust_toolchain_workspaces(config: Option<WorkspaceConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    
    for crate_config in &config.crates {
        println!("📦 Processing {}...", crate_config.display_name);

        let input_path = match crate_config.source_type.as_str() {
            "submodule" => config.submodules_dir.join(&crate_config.actual_name).join("Cargo.toml"),
            "crates.io" => config.submodules_dir.join("crates-io-cache").join(&crate_config.actual_name).join("Cargo.toml"),
            "local" => config.base_dir.join(&crate_config.actual_name).join("Cargo.toml"),
            _ => config.submodules_dir.join(&crate_config.actual_name).join("Cargo.toml"),
        };

        let output_path = config.base_dir.join(&crate_config.display_name).join("Cargo.toml");

        if !input_path.exists() {
            println!("❌ Not found: {}", input_path.display());
            continue;
        }

        // Read the original Cargo.toml
        let content = fs::read_to_string(&input_path)?;
        let doc = content.parse::<DocumentMut>()?;

        // Create a new minimal document with just package info
        let mut new_doc = DocumentMut::new();
        
        if let Some(package) = doc.get("package") {
            new_doc["package"] = package.clone();
        }

        // Add workspace section with edition
        new_doc["workspace"] = toml_edit::table();
        new_doc["workspace"]["package"]["edition"] = toml_edit::value(config.edition.clone());

        // Write the minimal Cargo.toml
        fs::create_dir_all(config.base_dir.join(&crate_config.display_name).join("src"))?;
        fs::write(&output_path, new_doc.to_string())?;

        // Create minimal lib.rs
        let lib_content = format!(
            "//! Minimal implementation for cargo-vendormod processing\n//! This will be replaced with actual vendored code\n\npub fn placeholder() -> &'static str {{\n    \"{} - vendored by cargo-vendormod\"\n}}",
            crate_config.display_name
        );
        fs::write(
            config.base_dir.join(&crate_config.display_name).join("src").join("lib.rs"),
            lib_content
        )?;

        println!("✅ Created workspace for {}", crate_config.display_name);
    }

    Ok(())
}

/// Fix edition configuration in existing workspaces
pub fn fix_edition_in_existing_workspaces(base_dir: &Path, edition: &str) -> Result<()> {
    let entries = fs::read_dir(base_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_dir() {
            continue;
        }
        
        let cargo_toml_path = path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            continue;
        }
        
        let content = fs::read_to_string(&cargo_toml_path)?;
        let mut doc = content.parse::<DocumentMut>()?;
        
        // Check if edition is already set
        if doc.get("workspace").and_then(|w| w.get("package")).and_then(|p| p.get("edition")).is_some() {
            println!("✅ Edition already set for {}", path.display());
            continue;
        }
        
        // Add edition if missing
        if !doc.contains_key("workspace") {
            doc["workspace"] = toml_edit::table();
        }
        
        if !doc["workspace"].as_table_mut().unwrap().contains_key("package") {
            doc["workspace"]["package"] = toml_edit::table();
        }
        
        doc["workspace"]["package"]["edition"] = toml_edit::value(edition);
        
        fs::write(&cargo_toml_path, doc.to_string())?;
        println!("✅ Fixed edition for {}", path.display());
    }
    
    Ok(())
}