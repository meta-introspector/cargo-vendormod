use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use toml_edit::{Document, DocumentMut, Value};
use pathdiff::diff_paths;

use crate::context::AppContext;

pub fn cmd_patch(ctx: AppContext, _actions_plan: Vec<crate::actions::RepoAction>) -> Result<()> {
    println!("Generating/updating cargo patches...");
    
    if ctx.dry_run {
        println!("[DRY RUN] Would update .cargo/config.toml");
        return Ok(());
    }
    
    let cargo_config_dir = ctx.root_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir).context("Failed to create .cargo directory")?;
    let cargo_config_path = cargo_config_dir.join("config.toml");
    
    let mut config_doc = if cargo_config_path.exists() {
        let content = fs::read_to_string(&cargo_config_path)
            .with_context(|| format!("Failed to read {:?}", cargo_config_path))?;
        content
            .parse::<DocumentMut>()
            .context("Failed to parse .cargo/config.toml")?
    } else {
        DocumentMut::new()
    };
    
    let patch = config_doc
        .entry("patch")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("patch entry is not a table")?;
    
    let patch_crates_io = patch
        .entry("crates-io")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("crates-io entry is not a table")?;
    
    // Collect existing submodules
    let submodules_dir = &ctx.submodules_dir;
    let mut patch_count = 0;
    if let Ok(entries) = fs::read_dir(submodules_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let relative_path = diff_paths(&path, &ctx.root_dir)
                        .unwrap_or(path.clone());
                    let path_str = relative_path.to_string_lossy().to_string();
                    let dep_value = toml_edit::Item::Value(toml_edit::Value::from(path_str));
                    patch_crates_io.insert(dir_name, dep_value);
                    patch_count += 1;
                    if ctx.verbose {
                        println!("Added patch for {} -> {}", dir_name, relative_path.display());
                    }
                }
            }
        }
    }
    
    let config_content = config_doc.to_string();
    fs::write(&cargo_config_path, config_content)
        .with_context(|| format!("Failed to write to {:?}", cargo_config_path))?;
    
    println!("Successfully updated .cargo/config.toml with {} patches.", patch_count);
    Ok(())
}