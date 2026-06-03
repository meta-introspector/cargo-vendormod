use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use toml_edit::DocumentMut;

/// Zkperf integration module that can use either the built-in annotation
/// or call the external cargo-zkperf tool
pub struct ZkperfIntegrator {
    zkperf_path: Option<PathBuf>,
    use_builtin: bool,
}

impl ZkperfIntegrator {
    /// Create a new ZkperfIntegrator
    pub fn new(zkperf_path: Option<PathBuf>, use_builtin: bool) -> Self {
        Self { zkperf_path, use_builtin }
    }
    
    /// Apply zkperf annotations to a repository
    pub fn apply_zkperf_annotations(&self, repo_path: &Path) -> Result<()> {
        if self.use_builtin {
            self.apply_builtin_annotations(repo_path)
        } else if let Some(zkperf_path) = &self.zkperf_path {
            self.apply_external_zkperf(zkperf_path, repo_path)
        } else {
            Ok(())
        }
    }
    
    /// Apply built-in zkperf annotations (metadata-based approach)
    fn apply_builtin_annotations(&self, repo_path: &Path) -> Result<()> {
        let cargo_toml = repo_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&cargo_toml)
            .context(format!("Failed to read Cargo.toml for {}", repo_path.display()))?;
        
        let mut doc = content.parse::<DocumentMut>()
            .context(format!("Failed to parse Cargo.toml for {}", repo_path.display()))?;
        
        // Add zkperf configuration to package metadata
        let package_table = doc
            .entry("package")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("package entry is not a table")?;
        
        let metadata_table = package_table
            .entry("metadata")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("metadata entry is not a table")?;
        
        let zkperf_table = metadata_table
            .entry("zkperf")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("zkperf entry is not a table")?;
        
        // Add zkperf-specific annotations
        zkperf_table.insert("enabled", toml_edit::value(true));
        zkperf_table.insert("benchmark", toml_edit::value(true));
        zkperf_table.insert("profiling", toml_edit::value(true));
        
        // Write the updated Cargo.toml
        let updated_content = doc.to_string();
        fs::write(&cargo_toml, updated_content)
            .context(format!("Failed to write updated Cargo.toml for {}", repo_path.display()))?;
        
        Ok(())
    }
    
    /// Apply zkperf annotations using external cargo-zkperf tool
    fn apply_external_zkperf(&self, zkperf_path: &Path, repo_path: &Path) -> Result<()> {
        println!("Applying zkperf annotations using external tool: {}", zkperf_path.display());
        
        // Check if cargo-zkperf exists
        if !zkperf_path.exists() {
            println!("cargo-zkperf not found at {}, falling back to built-in annotations", zkperf_path.display());
            return self.apply_builtin_annotations(repo_path);
        }
        
        // Run cargo-zkperf annotate on the repository
        let output = Command::new(zkperf_path)
            .arg("annotate")
            .arg(repo_path)
            .output()
            .context("Failed to execute cargo-zkperf")?;
        
        if !output.status.success() {
            eprintln!("Warning: cargo-zkperf failed: {}", String::from_utf8_lossy(&output.stderr));
            println!("Falling back to built-in annotations");
            return self.apply_builtin_annotations(repo_path);
        }
        
        println!("Successfully applied zkperf annotations using external tool");
        Ok(())
    }
    
    /// Run zkperf audit on a repository
    pub fn run_zkperf_audit(&self, repo_path: &Path) -> Result<()> {
        if let Some(zkperf_path) = &self.zkperf_path {
            self.run_external_zkperf_audit(zkperf_path, repo_path)
        } else {
            println!("Zkperf audit requires external cargo-zkperf tool");
            Ok(())
        }
    }
    
    /// Run zkperf audit using external tool
    fn run_external_zkperf_audit(&self, zkperf_path: &Path, repo_path: &Path) -> Result<()> {
        if !zkperf_path.exists() {
            println!("cargo-zkperf not found at {}", zkperf_path.display());
            return Ok(());
        }
        
        println!("Running zkperf audit on {}...", repo_path.display());
        
        let output = Command::new(zkperf_path)
            .arg("audit")
            .arg(repo_path)
            .output()
            .context("Failed to execute cargo-zkperf audit")?;
        
        if output.status.success() {
            println!("Zkperf audit output:\n{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("Zkperf audit failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    /// Generate zkperf report for a repository
    pub fn generate_zkperf_report(&self, repo_path: &Path, report_path: &Path) -> Result<()> {
        if let Some(zkperf_path) = &self.zkperf_path {
            self.generate_external_zkperf_report(zkperf_path, repo_path, report_path)
        } else {
            println!("Zkperf report generation requires external cargo-zkperf tool");
            Ok(())
        }
    }
    
    /// Generate zkperf report using external tool
    fn generate_external_zkperf_report(&self, zkperf_path: &Path, repo_path: &Path, report_path: &Path) -> Result<()> {
        if !zkperf_path.exists() {
            println!("cargo-zkperf not found at {}", zkperf_path.display());
            return Ok(());
        }
        
        println!("Generating zkperf report for {}...", repo_path.display());
        
        let output = Command::new(zkperf_path)
            .arg("report")
            .arg(repo_path)
            .arg("--output")
            .arg(report_path)
            .output()
            .context("Failed to execute cargo-zkperf report")?;
        
        if output.status.success() {
            println!("Zkperf report generated at {}", report_path.display());
        } else {
            eprintln!("Zkperf report generation failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
}