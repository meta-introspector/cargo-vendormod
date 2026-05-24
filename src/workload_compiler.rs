use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::collections::{HashMap, HashSet};
use toml_edit::{Document, DocumentMut};
use serde::{Serialize, Deserialize};

use crate::workload_optimizer::{WorkloadOptimizer, WorkloadPerformanceData};
use crate::zkperf_integration::ZkperfIntegrator;

/// Workload-specific compilation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCompilationConfig {
    pub workload_name: String,
    pub binary_name: String,
    pub test_case: Option<String>,
    pub entry_point: String,
    pub dependencies: HashSet<String>,
    pub zkperf_workload: PathBuf,
    pub optimized_crates: HashMap<String, WorkloadPerformanceData>,
    pub cargo_profile: String,
    pub compilation_flags: Vec<String>,
}

/// Workload compiler that creates isolated compilations for each binary/test case
pub struct WorkloadCompiler {
    base_workspace: PathBuf,
    workloads_dir: PathBuf,
    zkperf_integrator: ZkperfIntegrator,
    aggressive_optimization: bool,
}

impl WorkloadCompiler {
    /// Create a new WorkloadCompiler
    pub fn new(base_workspace: PathBuf, workloads_dir: PathBuf, zkperf_path: Option<PathBuf>, aggressive: bool) -> Self {
        Self {
            base_workspace,
            workloads_dir,
            zkperf_integrator: ZkperfIntegrator::new(zkperf_path, true),
            aggressive_optimization: aggressive,
        }
    }
    
    /// Discover all binaries and test cases in the workspace
    pub fn discover_workloads(&self) -> Result<Vec<WorkloadCompilationConfig>> {
        println!("Discovering workloads in {}...", self.base_workspace.display());
        
        let mut workloads = Vec::new();
        
        // Find all Cargo.toml files that contain binaries or tests
        let cargo_files = self.find_all_cargo_files(&self.base_workspace)?;
        
        for cargo_file in cargo_files {
            let package_workloads = self.analyze_cargo_file(&cargo_file)?;
            workloads.extend(package_workloads);
        }
        
        println!("Found {} potential workloads", workloads.len());
        Ok(workloads)
    }
    
    /// Find all Cargo.toml files in the workspace
    fn find_all_cargo_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut cargo_files = Vec::new();
        
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.file_name() == Some("Cargo.toml".as_ref()) {
                    cargo_files.push(path);
                }
                
                if path.is_dir() {
                    let mut sub_files = self.find_all_cargo_files(&path)?;
                    cargo_files.append(&mut sub_files);
                }
            }
        }
        
        Ok(cargo_files)
    }
    
    /// Analyze a Cargo.toml file for binaries and test cases
    fn analyze_cargo_file(&self, cargo_file: &Path) -> Result<Vec<WorkloadCompilationConfig>> {
        let mut workloads = Vec::new();
        
        let content = fs::read_to_string(cargo_file)
            .context(format!("Failed to read {}", cargo_file.display()))?;
        
        let doc = content.parse::<DocumentMut>()
            .context(format!("Failed to parse {}", cargo_file.display()))?;
        
        // Get package name
        let package_name = doc["package"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        // Check for binary targets
        if let Some(bins) = doc.get("bin") {
            if let toml_edit::Item::Value(toml_edit::Value::Array(bins_array)) = bins {
                for bin_item in bins_array.iter() {
                    if let toml_edit::Value::InlineTable(bin_table) = bin_item {
                        if let Some(name) = bin_table.get("name").and_then(|v| v.as_str()) {
                            let workload = self.create_binary_workload(&package_name, name, cargo_file)?;
                            workloads.push(workload);
                        }
                    }
                }
            }
        }
        
        // Check for single binary in package
        if doc["package"]["name"].as_str() == Some(package_name.as_str()) {
            // This is a binary package
            let workload = self.create_binary_workload(&package_name, &package_name, cargo_file)?;
            workloads.push(workload);
        }
        
        // Look for test cases (simplified - in real implementation would be more sophisticated)
        if let Some(features) = doc["features"].as_table() {
            for (feature_name, _) in features.iter() {
                if feature_name.starts_with("test-") || feature_name.ends_with("-test") {
                    let test_workload = self.create_test_workload(&package_name, feature_name, cargo_file)?;
                    workloads.push(test_workload);
                }
            }
        }
        
        Ok(workloads)
    }
    
    /// Create a workload configuration for a binary
    fn create_binary_workload(&self, package_name: &str, binary_name: &str, cargo_file: &Path) -> Result<WorkloadCompilationConfig> {
        let workload_name = format!("{}-{}", package_name, binary_name);
        let zkperf_workload = self.workloads_dir.join(&workload_name).with_extension("zkperf");
        
        Ok(WorkloadCompilationConfig {
            workload_name: workload_name.clone(),
            binary_name: binary_name.to_string(),
            test_case: None,
            entry_point: format!("src/bin/{}.rs", binary_name),
            dependencies: HashSet::new(),
            zkperf_workload,
            optimized_crates: HashMap::new(),
            cargo_profile: format!("{}-optimized", workload_name),
            compilation_flags: vec![
                "--release".to_string(),
                "--profile".to_string(),
                format!("{}-optimized", workload_name),
            ],
        })
    }
    
    /// Create a workload configuration for a test case
    fn create_test_workload(&self, package_name: &str, test_name: &str, cargo_file: &Path) -> Result<WorkloadCompilationConfig> {
        let workload_name = format!("{}-{}", package_name, test_name);
        let zkperf_workload = self.workloads_dir.join(&workload_name).with_extension("zkperf");
        
        Ok(WorkloadCompilationConfig {
            workload_name: workload_name.clone(),
            binary_name: format!("test-{}", test_name.replace("test-", "")),
            test_case: Some(test_name.to_string()),
            entry_point: "tests/.rs".to_string(), // Simplified
            dependencies: HashSet::new(),
            zkperf_workload,
            optimized_crates: HashMap::new(),
            cargo_profile: format!("{}-optimized", workload_name),
            compilation_flags: vec![
                "--test".to_string(),
                "--release".to_string(),
                "--features".to_string(),
                test_name.to_string(),
                "--profile".to_string(),
                format!("{}-optimized", workload_name),
            ],
        })
    }
    
    /// Generate zkperf workload for a specific binary/test case
    pub fn generate_zkperf_workload(&self, config: &WorkloadCompilationConfig) -> Result<()> {
        println!("Generating zkperf workload for {}...", config.workload_name);
        
        // Create workload directory
        fs::create_dir_all(&self.workloads_dir)
            .context("Failed to create workloads directory")?;
        
        // For now, create a placeholder workload file
        // In real implementation, this would run the binary/test and capture zkperf data
            let workload_content = format!(
                "# Zkperf Workload: {}\n\nbinary: {}\ntest_case: {:?}\nentry_point: {}\ndependencies:\n",
                config.workload_name, config.binary_name, config.test_case, config.entry_point
            );
        
        for dep in &config.dependencies {
            workload_content.push_str(&format!("  - {}\n", dep));
        }
        
        fs::write(&config.zkperf_workload, workload_content)
            .context("Failed to write zkperf workload file")?;
        
        Ok(())
    }
    
    /// Create isolated compilation for a workload
    pub fn create_isolated_compilation(&self, config: &WorkloadCompilationConfig) -> Result<()> {
        println!("Creating isolated compilation for {}...", config.workload_name);
        
        let workload_dir = self.workloads_dir.join(&config.workload_name);
        fs::create_dir_all(&workload_dir)
            .context("Failed to create workload directory")?;
        
        // Create a minimal Cargo.toml for this workload
        self.create_workload_cargo_toml(&workload_dir, config)?;
        
        // Create optimized cargo profile
        self.create_workload_cargo_profile(&workload_dir, config)?;
        
        // Copy necessary source files
        self.copy_workload_source_files(&workload_dir, config)?;
        
        Ok(())
    }
    
    /// Create workload-specific Cargo.toml
    fn create_workload_cargo_toml(&self, workload_dir: &Path, config: &WorkloadCompilationConfig) -> Result<()> {
        let cargo_toml = workload_dir.join("Cargo.toml");
        
        let mut doc = DocumentMut::new();
        
        // Package section
        let package_table = doc
            .entry("package")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("package entry is not a table")?;
        
        package_table.insert("name", toml_edit::value(&config.workload_name));
        package_table.insert("version", toml_edit::value("0.1.0"));
        package_table.insert("edition", toml_edit::value("2021"));
        
        // Add zkperf metadata
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
        
        zkperf_table.insert("workload", toml_edit::value(&config.workload_name));
        zkperf_table.insert("binary", toml_edit::value(&config.binary_name));
        if let Some(test_case) = &config.test_case {
            zkperf_table.insert("test_case", toml_edit::value(test_case));
        }
        
        // Binary section - create as array of inline tables
        let bin_items = vec![
            toml_edit::Value::InlineTable({
                let mut table = toml_edit::InlineTable::new();
                table.insert("name", config.binary_name.clone().into());
                table.insert("path", config.entry_point.clone().into());
                table
            })
        ];
        
        let bin_array = bin_items.into_iter().collect::<toml_edit::Value>();
        doc.insert("bin", bin_array);
        
        // Write the Cargo.toml
        let content = doc.to_string();
        fs::write(&cargo_toml, content)
            .context("Failed to write workload Cargo.toml")?;
        
        Ok(())
    }
    
    /// Create workload-specific cargo profile
    fn create_workload_cargo_profile(&self, workload_dir: &Path, config: &WorkloadCompilationConfig) -> Result<()> {
        let cargo_config_dir = workload_dir.join(".cargo");
        fs::create_dir_all(&cargo_config_dir)
            .context("Failed to create .cargo directory")?;
        
        let config_path = cargo_config_dir.join("config.toml");
        
        let mut doc = DocumentMut::new();
        
        // Create the optimized profile
        let profile_table = doc
            .entry("profile")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("profile entry is not a table")?;
        
        let optimized_profile = profile_table
            .entry(&config.cargo_profile)
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("profile entry is not a table")?;
        
        // Aggressive optimizations for this specific workload
        optimized_profile.insert("inherits", toml_edit::value("release"));
        optimized_profile.insert("opt-level", toml_edit::value("3"));
        optimized_profile.insert("lto", toml_edit::value("true"));
        optimized_profile.insert("codegen-units", toml_edit::value("1"));
        optimized_profile.insert("panic", toml_edit::value("abort"));
        optimized_profile.insert("incremental", toml_edit::value(false));
        
        // Workload-specific optimizations
        if self.aggressive_optimization {
            optimized_profile.insert("overflow-checks", toml_edit::value(false));
        }
        
        // Write the config
        let content = doc.to_string();
        fs::write(&config_path, content)
            .context("Failed to write cargo config")?;
        
        Ok(())
    }
    
    /// Copy source files needed for the workload
    fn copy_workload_source_files(&self, workload_dir: &Path, config: &WorkloadCompilationConfig) -> Result<()> {
        println!("Copying source files for {}...", config.workload_name);
        
        // Create source directory structure
        let src_dir = workload_dir.join("src");
        fs::create_dir_all(&src_dir)
            .context("Failed to create src directory")?;
        
        // For binaries, copy the main file
        if config.entry_point.ends_with(".rs") {
            let source_file = self.base_workspace.join(&config.entry_point);
            if source_file.exists() {
                let dest_file = workload_dir.join(&config.entry_point);
                fs::create_dir_all(dest_file.parent().unwrap())
                    .context("Failed to create parent directories")?;
                fs::copy(&source_file, &dest_file)
                    .context("Failed to copy source file")?;
            }
        }
        
        Ok(())
    }
    
    /// Compile a specific workload with optimizations
    pub fn compile_workload(&self, config: &WorkloadCompilationConfig, dry_run: bool) -> Result<()> {
        println!("Compiling workload: {}...", config.workload_name);
        
        if dry_run {
            println!("--- DRY RUN MODE ---");
            println!("Would compile with: {:?}", config.compilation_flags);
            return Ok(());
        }
        
        let workload_dir = self.workloads_dir.join(&config.workload_name);
        
        // Run cargo build with workload-specific flags
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .current_dir(&workload_dir);
        
        for flag in &config.compilation_flags {
            cmd.arg(flag);
        }
        
        let output = cmd.output()
            .context("Failed to execute cargo build")?;
        
        if output.status.success() {
            println!("Successfully compiled {}:", config.workload_name);
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("Compilation failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Workload compilation failed");
        }
        
        Ok(())
    }
    
    /// Create workloads for all binaries and test cases
    pub fn create_all_workloads(&self) -> Result<()> {
        println!("Creating workloads for all binaries and test cases...");
        
        // Discover all workloads
        let workloads = self.discover_workloads()?;
        
        // Create isolated compilations for each workload
        for (i, config) in workloads.iter().enumerate() {
            println!("\nProcessing workload {}/{}: {}", i + 1, workloads.len(), config.workload_name);
            
            // Generate zkperf workload
            self.generate_zkperf_workload(config)?;
            
            // Create isolated compilation
            self.create_isolated_compilation(config)?;
            
            // Compile the workload
            self.compile_workload(config, false)?;
        }
        
        println!("\nCreated {} workload-specific compilations", workloads.len());
        Ok(())
    }
    
    /// Generate a workload matrix showing all workloads and their dependencies
    pub fn generate_workload_matrix(&self, output_path: &Path) -> Result<()> {
        println!("Generating workload matrix...");
        
        let workloads = self.discover_workloads()?;
        
        let mut matrix_content = String::new();
        matrix_content.push_str("# Workload Matrix\n\n");
        matrix_content.push_str(&format!("Total Workloads: {}\n\n", workloads.len()));
        
        matrix_content.push_str("| Workload Name | Binary | Test Case | Profile | Dependencies |\n");
        matrix_content.push_str("|--------------|--------|-----------|---------|--------------|\n");
        
        for config in &workloads {
            matrix_content.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                config.workload_name,
                config.binary_name,
                config.test_case.as_deref().unwrap_or("-"),
                config.cargo_profile,
                config.dependencies.len()
            ));
        }
        
        fs::write(output_path, matrix_content)
            .context("Failed to write workload matrix")?;
        
        println!("Workload matrix generated at {}", output_path.display());
        Ok(())
    }
    
    /// Create a workload-specific compilation with full optimization pipeline
    pub fn create_optimized_workload(&self, workload_name: &str) -> Result<WorkloadCompilationConfig> {
        println!("Creating fully optimized workload: {}...", workload_name);
        
        // Step 1: Discover the specific workload
        let workloads = self.discover_workloads()?;
        let config = workloads.into_iter()
            .find(|w| w.workload_name == workload_name || w.binary_name == workload_name)
            .context("Workload not found")?;
        
        // Step 2: Generate zkperf workload
        self.generate_zkperf_workload(&config)?;
        
        // Step 3: Create isolated compilation
        self.create_isolated_compilation(&config)?;
        
        // Step 4: Run workload optimizer
        let mut optimizer = WorkloadOptimizer::new(
            Some(PathBuf::from("/home/mdupont/projects/cargo/submodules/zkperf/cargo-zkperf")),
            self.aggressive_optimization
        );
        
        let workload_dir = self.workloads_dir.join(&config.workload_name);
        optimizer.analyze_workload(&workload_dir, &workload_dir)?;
        optimizer.apply_optimizations(&workload_dir, &self.base_workspace, false)?;
        
        // Step 5: Update config with optimization data
        let mut optimized_config = config.clone();
        optimized_config.optimized_crates = optimizer.performance_data;
        
        // Step 6: Generate optimization report
        let report_path = self.workloads_dir.join(&config.workload_name).with_extension("report.md");
        optimizer.generate_optimization_report(&report_path)?;
        
        // Step 7: Compile the optimized workload
        self.compile_workload(&optimized_config, false)?;
        
        Ok(optimized_config)
    }
    
    /// Compile all workloads with optimizations
    pub fn compile_all_workloads(&self) -> Result<()> {
        println!("Compiling all workloads with optimizations...");
        
        let workloads = self.discover_workloads()?;
        
        for (i, config) in workloads.iter().enumerate() {
            println!("\nCompiling workload {}/{}: {}", i + 1, workloads.len(), config.workload_name);
            self.create_optimized_workload(&config.workload_name)?;
        }
        
        println!("\nAll workloads compiled successfully!");
        Ok(())
    }
}