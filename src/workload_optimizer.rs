use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};
use toml_edit::{Document, DocumentMut};

use crate::zkperf_integration::ZkperfIntegrator;

/// Workload performance data used for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPerformanceData {
    pub crate_name: String,
    pub functions_used: HashSet<String>,
    pub modules_used: HashSet<String>,
    pub performance_hotspots: Vec<PerformanceHotspot>,
    pub unused_modules: HashSet<String>,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}

/// Performance hotspot identified by zkperf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHotspot {
    pub function_name: String,
    pub execution_time_ms: f64,
    pub call_count: usize,
    pub risk_score: u32,
    pub file_location: String,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub recommendation_type: String,
    pub target: String,
    pub estimated_impact: String,
    pub confidence: f32,
}

/// Workload optimizer that uses zkperf data for tree-shaking
pub struct WorkloadOptimizer {
    zkperf_integrator: ZkperfIntegrator,
    performance_data: HashMap<String, WorkloadPerformanceData>,
    aggressive_optimization: bool,
}

impl WorkloadOptimizer {
    /// Create a new WorkloadOptimizer
    pub fn new(zkperf_path: Option<PathBuf>, aggressive: bool) -> Self {
        Self {
            zkperf_integrator: ZkperfIntegrator::new(zkperf_path, true),
            performance_data: HashMap::new(),
            aggressive_optimization: aggressive,
        }
    }
    
    /// Analyze workload and generate performance data
    pub fn analyze_workload(&mut self, workspace_path: &Path, fork_dir: &Path) -> Result<()> {
        println!("Analyzing workload performance...");
        
        // Step 1: Run zkperf audit on all crates
        self.run_zkperf_analysis(fork_dir)?;
        
        // Step 2: Analyze cargo dependency tree
        self.analyze_dependency_tree(workspace_path)?;
        
        // Step 3: Generate optimization recommendations
        self.generate_optimization_recommendations();
        
        Ok(())
    }
    
    /// Run zkperf analysis on all forked crates
    fn run_zkperf_analysis(&mut self, fork_dir: &Path) -> Result<()> {
        println!("Running zkperf analysis on forked crates...");
        
        if let Ok(entries) = fs::read_dir(fork_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !path.extension().map_or(false, |ext| ext == "git") {
                    if let Some(crate_name) = path.file_name().and_then(|n| n.to_str()) {
                        self.analyze_crate_with_zkperf(&path, crate_name)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze a single crate with zkperf
    fn analyze_crate_with_zkperf(&mut self, crate_path: &Path, crate_name: &str) -> Result<()> {
        println!("Analyzing {} with zkperf...", crate_name);
        
        // Generate zkperf report
        let report_path = crate_path.join("zkperf_report.json");
        self.zkperf_integrator.generate_zkperf_report(crate_path, &report_path)?;
        
        // Parse the report (simplified - in real implementation would parse actual JSON)
        let performance_data = self.parse_zkperf_report(&report_path, crate_name)?;
        
        self.performance_data.insert(crate_name.to_string(), performance_data);
        
        Ok(())
    }
    
    /// Parse zkperf report (simplified mock implementation)
    fn parse_zkperf_report(&self, report_path: &Path, crate_name: &str) -> Result<WorkloadPerformanceData> {
        // In a real implementation, this would parse the actual JSON report
        // For now, we'll create a mock response
        
        let mut functions_used = HashSet::new();
        functions_used.insert("main".to_string());
        functions_used.insert("process_transaction".to_string());
        functions_used.insert("verify_signature".to_string());
        
        let mut modules_used = HashSet::new();
        modules_used.insert("core".to_string());
        modules_used.insert("transaction".to_string());
        modules_used.insert("crypto".to_string());
        
        let mut unused_modules = HashSet::new();
        unused_modules.insert("test_utils".to_string());
        unused_modules.insert("benchmarking".to_string());
        unused_modules.insert("legacy_code".to_string());
        
        let hotspots = vec![
            PerformanceHotspot {
                function_name: "verify_signature".to_string(),
                execution_time_ms: 45.2,
                call_count: 128,
                risk_score: 85,
                file_location: "src/crypto.rs:42".to_string(),
            },
            PerformanceHotspot {
                function_name: "process_transaction".to_string(),
                execution_time_ms: 12.8,
                call_count: 64,
                risk_score: 68,
                file_location: "src/transaction.rs:18".to_string(),
            },
        ];
        
        let recommendations = vec![
            OptimizationRecommendation {
                recommendation_type: "tree-shaking".to_string(),
                target: "test_utils module".to_string(),
                estimated_impact: "Reduce binary size by ~12%".to_string(),
                confidence: 0.95,
            },
            OptimizationRecommendation {
                recommendation_type: "code-inlining".to_string(),
                target: "verify_signature function".to_string(),
                estimated_impact: "Improve performance by ~15%".to_string(),
                confidence: 0.87,
            },
        ];
        
        Ok(WorkloadPerformanceData {
            crate_name: crate_name.to_string(),
            functions_used,
            modules_used,
            performance_hotspots: hotspots,
            unused_modules,
            optimization_recommendations: recommendations,
        })
    }
    
    /// Analyze cargo dependency tree
    fn analyze_dependency_tree(&mut self, workspace_path: &Path) -> Result<()> {
        println!("Analyzing cargo dependency tree...");
        
        // Run cargo tree to get dependency information
        let output = Command::new("cargo")
            .arg("tree")
            .arg("--format")
            .arg("{p} {f}")
            .current_dir(workspace_path)
            .output();
        
        if let Ok(out) = output {
            if out.status.success() {
                let tree_output = String::from_utf8_lossy(&out.stdout);
                println!("Dependency tree:\n{}", tree_output);
                // In real implementation, parse this to build dependency graph
            }
        }
        
        Ok(())
    }
    
    /// Generate optimization recommendations
    fn generate_optimization_recommendations(&mut self) {
        println!("Generating optimization recommendations...");
        
        for (crate_name, data) in &mut self.performance_data {
            // Add global recommendations based on unused modules
            for unused_module in &data.unused_modules {
                data.optimization_recommendations.push(OptimizationRecommendation {
                    recommendation_type: "module-removal".to_string(),
                    target: format!("{}/{}", crate_name, unused_module),
                    estimated_impact: "Reduce compile time and binary size".to_string(),
                    confidence: 0.90,
                });
            }
            
            // Add recommendations for performance hotspots
            for hotspot in &data.performance_hotspots {
                if hotspot.risk_score > 70 {
                    data.optimization_recommendations.push(OptimizationRecommendation {
                        recommendation_type: "performance-optimization".to_string(),
                        target: format!("{}:{}", hotspot.function_name, hotspot.file_location),
                        estimated_impact: format!("Potential {}% improvement", (hotspot.risk_score - 50) as f32 / 2.0),
                        confidence: 0.75 + (hotspot.risk_score as f32 / 200.0),
                    });
                }
            }
        }
    }
    
    /// Apply optimizations to remove unused code
    pub fn apply_optimizations(&self, fork_dir: &Path, workspace_path: &Path, dry_run: bool) -> Result<()> {
        println!("Applying workload optimizations...");
        
        if dry_run {
            println!("--- DRY RUN MODE ---");
            println!("Would apply the following optimizations:");
            self.print_optimization_summary();
            return Ok(());
        }
        
        // Apply optimizations to each crate
        for (crate_name, data) in &self.performance_data {
            let crate_path = fork_dir.join(crate_name);
            if crate_path.exists() {
                self.optimize_crate(&crate_path, data)?;
            }
        }
        
        // Generate optimized cargo configuration
        self.generate_optimized_cargo_config(workspace_path)?;
        
        Ok(())
    }
    
    /// Print optimization summary
    fn print_optimization_summary(&self) {
        for (crate_name, data) in &self.performance_data {
            println!("\nCrate: {}", crate_name);
            println!("  Functions used: {}", data.functions_used.len());
            println!("  Modules used: {}", data.modules_used.len());
            println!("  Unused modules: {}", data.unused_modules.len());
            println!("  Performance hotspots: {}", data.performance_hotspots.len());
            println!("  Recommendations: {}", data.optimization_recommendations.len());
            
            for rec in &data.optimization_recommendations {
                println!("    - {}: {} (confidence: {:.1}%)", 
                    rec.recommendation_type, rec.target, rec.confidence * 100.0);
            }
        }
    }
    
    /// Optimize a single crate
    fn optimize_crate(&self, crate_path: &Path, data: &WorkloadPerformanceData) -> Result<()> {
        println!("Optimizing crate: {}", crate_path.display());
        
        // 1. Update Cargo.toml with optimization settings
        self.optimize_cargo_toml(crate_path)?;
        
        // 2. Remove unused modules (comment them out)
        self.remove_unused_modules(crate_path, &data.unused_modules)?;
        
        // 3. Add optimization attributes to hotspot functions
        self.optimize_hotspot_functions(crate_path, &data.performance_hotspots)?;
        
        Ok(())
    }
    
    /// Optimize Cargo.toml for the crate
    fn optimize_cargo_toml(&self, crate_path: &Path) -> Result<()> {
        let cargo_toml = crate_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&cargo_toml)
            .context(format!("Failed to read Cargo.toml at {}", cargo_toml.display()))?;
        
        let mut doc = content.parse::<DocumentMut>()
            .context(format!("Failed to parse Cargo.toml at {}", cargo_toml.display()))?;
        
        // Add optimization profile
        let profile_table = doc
            .entry("profile")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("profile entry is not a table")?;
        
        let release_table = profile_table
            .entry("release")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("release profile is not a table")?;
        
        // Add optimization settings
        release_table.insert("opt-level", toml_edit::value("3"));
        release_table.insert("lto", toml_edit::value("true"));
        release_table.insert("codegen-units", toml_edit::value("1"));
        release_table.insert("panic", toml_edit::value("abort"));
        
        // Add zkperf-specific optimizations
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
        
        zkperf_table.insert("optimized", toml_edit::value(true));
        zkperf_table.insert("tree-shaking", toml_edit::value(true));
        zkperf_table.insert("aggressive-optimization", toml_edit::value(self.aggressive_optimization));
        
        // Write the updated Cargo.toml
        let updated_content = doc.to_string();
        fs::write(&cargo_toml, updated_content)
            .context(format!("Failed to write updated Cargo.toml at {}", cargo_toml.display()))?;
        
        Ok(())
    }
    
    /// Remove unused modules by commenting them out
    fn remove_unused_modules(&self, crate_path: &Path, unused_modules: &HashSet<String>) -> Result<()> {
        println!("Removing unused modules: {:?}", unused_modules);
        
        // Find all Rust source files
        let mut rust_files = Vec::new();
        self.find_rust_files(crate_path, &mut rust_files)?;
        
        for file_path in rust_files {
            let content = fs::read_to_string(&file_path)
                .context(format!("Failed to read {}", file_path.display()))?;
            
            let mut modified_content = content;
            
            // Comment out module declarations for unused modules
            for module in unused_modules {
                let module_decl = format!("mod {};", module);
                let commented_decl = format!("// zkperf-optimized: mod {};", module);
                modified_content = modified_content.replace(&module_decl, &commented_decl);
                
                // Also comment out any use statements
                let use_stmt = format!("use {}::", module);
                let commented_use = format!("// zkperf-optimized: use {}::", module);
                modified_content = modified_content.replace(&use_stmt, &commented_use);
            }
            
            // Write back if changes were made
            if modified_content != content {
                fs::write(&file_path, modified_content)
                    .context(format!("Failed to write optimized {}", file_path.display()))?;
                println!("Optimized {}", file_path.display());
            }
        }
        
        Ok(())
    }
    
    /// Find all Rust source files in a directory
    fn find_rust_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.find_rust_files(&path, files)?;
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Optimize hotspot functions with attributes
    fn optimize_hotspot_functions(&self, crate_path: &Path, hotspots: &[PerformanceHotspot]) -> Result<()> {
        println!("Optimizing {} hotspot functions...", hotspots.len());
        
        // Find all Rust source files
        let mut rust_files = Vec::new();
        self.find_rust_files(crate_path, &mut rust_files)?;
        
        for file_path in rust_files {
            let content = fs::read_to_string(&file_path)
                .context(format!("Failed to read {}", file_path.display()))?;
            
            let mut modified_content = content;
            
            // Add optimization attributes to hotspot functions
            for hotspot in hotspots {
                if hotspot.risk_score > 60 {
                    let attr = format!("#[zkperf::optimize] // Hotspot: {}ms, {} calls", 
                        hotspot.execution_time_ms, hotspot.call_count);
                    let func_decl = format!("fn {}", hotspot.function_name);
                    
                    // Insert the attribute before the function
                    if let Some(pos) = modified_content.find(&func_decl) {
                        let before_func = &modified_content[..pos];
                        let after_func = &modified_content[pos..];
                        modified_content = format!("{}{}\n{}", before_func, attr, after_func);
                    }
                }
            }
            
            // Write back if changes were made
            if modified_content != content {
                fs::write(&file_path, modified_content)
                    .context(format!("Failed to write optimized {}", file_path.display()))?;
                println!("Optimized hotspots in {}", file_path.display());
            }
        }
        
        Ok(())
    }
    
    /// Generate optimized cargo configuration for the workspace
    fn generate_optimized_cargo_config(&self, workspace_path: &Path) -> Result<()> {
        println!("Generating optimized cargo configuration...");
        
        let cargo_config_dir = workspace_path.join(".cargo");
        fs::create_dir_all(&cargo_config_dir).context("Failed to create .cargo directory")?;
        
        let config_path = cargo_config_dir.join("config.toml");
        
        let mut config_doc = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {:?}", config_path))?;
            content.parse::<DocumentMut>()
                .context("Failed to parse .cargo/config.toml")?
        } else {
            DocumentMut::new()
        };
        
        // Add optimization profile settings
        let profile_table = config_doc
            .entry("profile")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("profile entry is not a table")?;
        
        let release_table = profile_table
            .entry("release")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("release profile is not a table")?;
        
        // Workload-specific optimizations
        release_table.insert("opt-level", toml_edit::value("3"));
        release_table.insert("lto", toml_edit::value("true"));
        release_table.insert("codegen-units", toml_edit::value("1"));
        release_table.insert("panic", toml_edit::value("abort"));
        release_table.insert("incremental", toml_edit::value(false));
        
        // Add zkperf-specific settings
        let zkperf_table = config_doc
            .entry("zkperf")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .context("zkperf entry is not a table")?;
        
        zkperf_table.insert("workload-optimized", toml_edit::value(true));
        zkperf_table.insert("tree-shaking", toml_edit::value(true));
        zkperf_table.insert("aggressive", toml_edit::value(self.aggressive_optimization));
        
        // Write the configuration
        let config_content = config_doc.to_string();
        fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write to {:?}", config_path))?;
        
        println!("Generated optimized cargo configuration");
        Ok(())
    }
    
    /// Generate a performance optimization report
    pub fn generate_optimization_report(&self, report_path: &Path) -> Result<()> {
        println!("Generating optimization report at {}", report_path.display());
        
        let report_content = self.format_optimization_report();
        fs::write(report_path, report_content)
            .context("Failed to write optimization report")?;
        
        Ok(())
    }
    
    /// Format the optimization report
    fn format_optimization_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Workload Optimization Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        for (crate_name, data) in &self.performance_data {
            report.push_str(&format!("## Crate: {}\n\n", crate_name));
            
            report.push_str(&format!("- **Functions used**: {}\n", data.functions_used.len()));
            report.push_str(&format!("- **Modules used**: {}\n", data.modules_used.len()));
            report.push_str(&format!("- **Unused modules**: {}\n", data.unused_modules.len()));
            report.push_str(&format!("- **Performance hotspots**: {}\n\n", data.performance_hotspots.len()));
            
            report.push_str("### Performance Hotspots:\n");
            for hotspot in &data.performance_hotspots {
                report.push_str(&format!("- **{}** ({}): {:.1}ms avg, {} calls, risk: {}\n",
                    hotspot.function_name, hotspot.file_location,
                    hotspot.execution_time_ms, hotspot.call_count, hotspot.risk_score));
            }
            
            report.push_str("\n### Optimization Recommendations:\n");
            for (i, rec) in data.optimization_recommendations.iter().enumerate() {
                report.push_str(&format!("{}. **{}**: {} (confidence: {:.0}%)\n",
                    i + 1, rec.recommendation_type, rec.target, rec.confidence * 100.0));
            }
            
            report.push_str("\n---\n\n");
        }
        
        report.push_str("## Summary\n\n");
        let total_crates = self.performance_data.len();
        let total_hotspots: usize = self.performance_data.values().map(|d| d.performance_hotspots.len()).sum();
        let total_unused: usize = self.performance_data.values().map(|d| d.unused_modules.len()).sum();
        let total_recommendations: usize = self.performance_data.values().map(|d| d.optimization_recommendations.len()).sum();
        
        report.push_str(&format!("- **Total crates analyzed**: {}\n", total_crates));
        report.push_str(&format!("- **Total performance hotspots**: {}\n", total_hotspots));
        report.push_str(&format!("- **Total unused modules**: {}\n", total_unused));
        report.push_str(&format!("- **Total recommendations**: {}\n", total_recommendations));
        
        if total_unused > 0 {
            let estimated_savings = (total_unused as f32 * 0.8) as usize; // Rough estimate
            report.push_str(&format!("- **Estimated binary size reduction**: ~{}%\n", estimated_savings));
        }
        
        report
    }
}