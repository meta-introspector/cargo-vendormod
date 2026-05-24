use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
// use reqwest::blocking::Client;
use regex::Regex;

/// Cargo tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoTool {
    pub name: String,
    pub crate_name: String,
    pub description: String,
    pub version: String,
    pub source: ToolSource,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub download_url: Option<String>,
    pub documentation_url: Option<String>,
    pub license: Option<String>,
    pub last_updated: Option<String>,
    pub compatibility: ToolCompatibility,
    pub integrated: bool,
}

/// Tool source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolSource {
    CratesIo,
    GitHub,
    Local,
    Custom,
}

/// Tool compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCompatibility {
    pub rust_version: Option<String>,
    pub platform_support: Vec<String>,
    pub dependency_conflicts: Vec<String>,
    pub integration_status: IntegrationStatus,
    pub compatibility_score: f32,
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrationStatus {
    NotIntegrated,
    Partial,
    Full,
    Native,
}

/// Tool search query
#[derive(Debug, Clone)]
pub struct ToolSearchQuery {
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub min_downloads: Option<u32>,
    pub rust_version: Option<String>,
    pub platform: Option<String>,
    pub limit: usize,
}

/// Cargo tool discovery and integration system
pub struct CargoToolDiscovery {
    // client: Client,
    github_token: Option<String>,
    local_tools_dir: PathBuf,
    integrated_tools: HashMap<String, CargoTool>,
    tool_cache: PathBuf,
}

impl CargoToolDiscovery {
    /// Create a new CargoToolDiscovery
    pub fn new(local_tools_dir: PathBuf, tool_cache: PathBuf, github_token: Option<String>) -> Self {
        Self {
            // client: Client::new(),
            github_token,
            local_tools_dir,
            integrated_tools: HashMap::new(),
            tool_cache,
        }
    }
    
    /// Search for tools on crates.io
    pub fn search_crates_io(&self, query: &ToolSearchQuery) -> Result<Vec<CargoTool>> {
        println!("Searching crates.io for analysis tools...");
        
        let mut tools = Vec::new();
        
        // Build search URL
        let mut url = "https://crates.io/api/v1/crates?page=1&per_page=100".to_string();
        
        if !query.keywords.is_empty() {
            let keywords = query.keywords.join("+");
            url.push_str(&format!("&q={}", keywords));
        }
        
        // In real implementation, this would make an HTTP request
        // For now, we'll return some known analysis tools
        
        let known_tools = vec![
            ("cargo-zkperf", "Zkperf analysis tool", "0.1.0"),
            ("cargo-perf", "Performance analysis", "1.2.3"),
            ("cargo-ebpf", "eBPF analysis", "0.4.5"),
            ("cargo-strace", "System call tracing", "2.1.0"),
            ("cargo-flamegraph", "Flamegraph generation", "0.5.2"),
            ("cargo-profiler", "Profiling tool", "1.0.0"),
            ("cargo-audit", "Security audit", "0.15.0"),
            ("cargo-deny", "Dependency checking", "0.12.4"),
        ];
        
        for (name, description, version) in known_tools {
            if query.keywords.is_empty() || 
               query.keywords.iter().any(|k| name.contains(k) || description.contains(k)) {
                
                let tool = CargoTool {
                    name: name.to_string(),
                    crate_name: name.replace("cargo-", ""),
                    description: description.to_string(),
                    version: version.to_string(),
                    source: ToolSource::CratesIo,
                    categories: vec!["analysis".to_string(), "performance".to_string()],
                    keywords: vec!["analysis".to_string(), "performance".to_string()],
                    download_url: Some(format!("https://crates.io/crates/{}", name)),
                    documentation_url: Some(format!("https://docs.rs/{}/{}", name, version)),
                    license: Some("MIT/Apache-2.0".to_string()),
                    last_updated: Some("2023-01-01".to_string()),
                    compatibility: ToolCompatibility {
                        rust_version: Some("1.56.0".to_string()),
                        platform_support: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
                        dependency_conflicts: Vec::new(),
                        integration_status: IntegrationStatus::Partial,
                        compatibility_score: 0.85,
                    },
                    integrated: false,
                };
                
                tools.push(tool);
            }
        }
        
        Ok(tools.into_iter().take(query.limit).collect())
    }
    
    /// Search for tools on GitHub
    pub fn search_github(&self, query: &ToolSearchQuery) -> Result<Vec<CargoTool>> {
        println!("Searching GitHub for analysis tools...");
        
        let mut tools = Vec::new();
        
        // Build GitHub search query
        let mut search_query = "cargo tool analysis".to_string();
        if !query.keywords.is_empty() {
            search_query.push_str(&format!(" {}", query.keywords.join(" ")));
        }
        
        // In real implementation, this would use GitHub API
        // For now, we'll return some known GitHub tools
        
        let github_tools = vec![
            ("mdupont/cargo-zkperf", "Advanced zkperf analysis", "0.2.0", "https://github.com/mdupont/cargo-zkperf"),
            ("rust-lang/cargo-perf", "Official perf tool", "1.0.0", "https://github.com/rust-lang/cargo-perf"),
            ("ebpf-rs/cargo-ebpf", "eBPF integration", "0.3.1", "https://github.com/ebpf-rs/cargo-ebpf"),
            ("strace/cargo-strace", "strace wrapper", "1.2.0", "https://github.com/strace/cargo-strace"),
            ("flamegraph-rs/flamegraph", "Flamegraph tool", "0.4.5", "https://github.com/flamegraph-rs/flamegraph"),
        ];
        
        for (repo, description, version, url) in github_tools {
            if query.keywords.is_empty() || 
               query.keywords.iter().any(|k| repo.contains(k) || description.contains(k)) {
                
                let tool = CargoTool {
                    name: repo.split('/').last().unwrap().to_string(),
                    crate_name: repo.split('/').last().unwrap().replace("cargo-", ""),
                    description: description.to_string(),
                    version: version.to_string(),
                    source: ToolSource::GitHub,
                    categories: vec!["analysis".to_string(), "github".to_string()],
                    keywords: vec!["analysis".to_string(), "github".to_string()],
                    download_url: Some(url.to_string()),
                    documentation_url: Some(format!("{}/README.md", url)),
                    license: Some("MIT".to_string()),
                    last_updated: Some("2023-06-01".to_string()),
                    compatibility: ToolCompatibility {
                        rust_version: Some("1.60.0".to_string()),
                        platform_support: vec!["linux".to_string()],
                        dependency_conflicts: Vec::new(),
                        integration_status: IntegrationStatus::Partial,
                        compatibility_score: 0.80,
                    },
                    integrated: false,
                };
                
                tools.push(tool);
            }
        }
        
        Ok(tools.into_iter().take(query.limit).collect())
    }
    
    /// Discover locally installed cargo tools
    pub fn discover_local_tools(&self) -> Result<Vec<CargoTool>> {
        println!("Discovering locally installed cargo tools...");
        
        let mut tools = Vec::new();
        
        // Check for cargo tools in PATH
        if let Ok(output) = Command::new("cargo").arg("--list").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse cargo tool list
                let tool_regex = Regex::new(r"cargo-(\w+)")?;
                for cap in tool_regex.captures_iter(&output_str) {
                    if let Some(tool_name) = cap.get(1) {
                        let tool_name = tool_name.as_str();
                        
                        // Check if tool is executable
                        if Command::new(format!("cargo-{}", tool_name))
                            .arg("--version")
                            .output()
                            .is_ok() {
                            
                            let tool = CargoTool {
                                name: format!("cargo-{}", tool_name),
                                crate_name: tool_name.to_string(),
                                description: format!("Locally installed {} tool", tool_name),
                                version: "unknown".to_string(),
                                source: ToolSource::Local,
                                categories: vec!["local".to_string()],
                                keywords: vec!["local".to_string(), "installed".to_string()],
                                download_url: None,
                                documentation_url: None,
                                license: None,
                                last_updated: None,
                                compatibility: ToolCompatibility {
                                    rust_version: None,
                                    platform_support: vec![std::env::consts::OS.to_string()],
                                    dependency_conflicts: Vec::new(),
                                    integration_status: IntegrationStatus::NotIntegrated,
                                    compatibility_score: 0.90, // Local tools assumed compatible
                                },
                                integrated: false,
                            };
                            
                            tools.push(tool);
                        }
                    }
                }
            }
        }
        
        // Also check our local tools directory
        if self.local_tools_dir.exists() {
            for entry in fs::read_dir(&self.local_tools_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("cargo-") {
                            let tool = CargoTool {
                                name: name.to_string(),
                                crate_name: name.replace("cargo-", ""),
                                description: format!("Local tool: {}", name),
                                version: "dev".to_string(),
                                source: ToolSource::Local,
                                categories: vec!["local".to_string(), "development".to_string()],
                                keywords: vec!["local".to_string()],
                                download_url: None,
                                documentation_url: None,
                                license: None,
                                last_updated: None,
                                compatibility: ToolCompatibility {
                                    rust_version: None,
                                    platform_support: vec![std::env::consts::OS.to_string()],
                                    dependency_conflicts: Vec::new(),
                                    integration_status: IntegrationStatus::NotIntegrated,
                                    compatibility_score: 0.75, // Dev tools may have lower compatibility
                                },
                                integrated: false,
                            };
                            
                            tools.push(tool);
                        }
                    }
                }
            }
        }
        
        Ok(tools)
    }
    
    /// Search all sources for tools
    pub fn search_all_sources(&self, query: &ToolSearchQuery) -> Result<Vec<CargoTool>> {
        println!("Searching all sources for analysis tools...");
        
        let mut all_tools = Vec::new();
        
        // Search crates.io
        let crates_tools = self.search_crates_io(query)?;
        all_tools.extend(crates_tools);
        
        // Search GitHub
        let github_tools = self.search_github(query)?;
        all_tools.extend(github_tools);
        
        // Discover local tools
        let local_tools = self.discover_local_tools()?;
        all_tools.extend(local_tools);
        
        // Remove duplicates
        let mut unique_tools = HashMap::new();
        for tool in all_tools {
            unique_tools.entry(tool.name.clone()).or_insert(tool);
        }
        
        Ok(unique_tools.into_values().collect())
    }
    
    /// Analyze tool compatibility with current system
    pub fn analyze_tool_compatibility(&self, tool: &CargoTool) -> Result<ToolCompatibility> {
        println!("Analyzing compatibility for {}...", tool.name);
        
        let mut compatibility = tool.compatibility.clone();
        
        // Check Rust version compatibility
        if let Some(required_version) = &tool.compatibility.rust_version {
            let current_version = self.get_rust_version()?;
            
            if !self.is_version_compatible(&current_version, required_version) {
                compatibility.compatibility_score *= 0.7;
                compatibility.dependency_conflicts.push(format!("rust_version: {} required, {} found", 
                    required_version, current_version));
            }
        }
        
        // Check platform support
        let current_platform = std::env::consts::OS;
        if !tool.compatibility.platform_support.contains(&current_platform.to_string()) {
            compatibility.compatibility_score *= 0.5;
            compatibility.dependency_conflicts.push(format!("platform: {} not supported", current_platform));
        }
        
        // Check for dependency conflicts with integrated tools
        for (_, integrated_tool) in &self.integrated_tools {
            if integrated_tool.name == tool.name {
                compatibility.compatibility_score *= 0.8;
                compatibility.dependency_conflicts.push(format!("already_integrated: {}", tool.name));
            }
        }
        
        // Update integration status based on compatibility
        if compatibility.compatibility_score >= 0.9 {
            compatibility.integration_status = IntegrationStatus::Full;
        } else if compatibility.compatibility_score >= 0.7 {
            compatibility.integration_status = IntegrationStatus::Partial;
        } else if compatibility.compatibility_score >= 0.5 {
            compatibility.integration_status = IntegrationStatus::Partial;
        } else {
            compatibility.integration_status = IntegrationStatus::NotIntegrated;
        }
        
        Ok(compatibility)
    }
    
    /// Get current Rust version
    fn get_rust_version(&self) -> Result<String> {
        let output = Command::new("rustc").arg("--version").output()?;
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = version_str.split_whitespace().nth(1) {
                return Ok(version.to_string());
            }
        }
        Ok("unknown".to_string())
    }
    
    /// Check if versions are compatible
    fn is_version_compatible(&self, current: &str, required: &str) -> bool {
        // Simple version comparison - in real implementation would parse properly
        let current_major: u32 = current.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let required_major: u32 = required.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
        
        current_major >= required_major
    }
    
    /// Integrate a tool into the system
    pub fn integrate_tool(&mut self, mut tool: CargoTool) -> Result<CargoTool> {
        println!("Integrating tool: {}...", tool.name);
        
        // Analyze compatibility
        let compatibility = self.analyze_tool_compatibility(&tool)?;
        tool.compatibility = compatibility;
        
        // Download/install if needed
        if !tool.integrated {
            match tool.source {
                ToolSource::CratesIo => {
                    self.install_from_crates_io(&tool)?;
                }
                ToolSource::GitHub => {
                    self.install_from_github(&tool)?;
                }
                ToolSource::Local => {
                    // Already local, just verify
                    self.verify_local_tool(&tool)?;
                }
                ToolSource::Custom => {
                    // Custom installation
                    self.install_custom_tool(&tool)?;
                }
            }
            
            tool.integrated = true;
        }
        
        // Add to integrated tools
        self.integrated_tools.insert(tool.name.clone(), tool.clone());
        
        Ok(tool)
    }
    
    /// Install tool from crates.io
    fn install_from_crates_io(&self, tool: &CargoTool) -> Result<()> {
        println!("Installing {} from crates.io...", tool.name);
        
        // In real implementation, this would run cargo install
        println!("Would run: cargo install {}", tool.name);
        
        // Create tool cache directory
        fs::create_dir_all(&self.tool_cache)?;
        
        // Mark as installed (simulated)
        let install_marker = self.tool_cache.join(format!("{}.installed", tool.name));
        fs::write(install_marker, "installed")?;
        
        Ok(())
    }
    
    /// Install tool from GitHub
    fn install_from_github(&self, tool: &CargoTool) -> Result<()> {
        println!("Installing {} from GitHub...", tool.name);
        
        // In real implementation, this would clone and build from GitHub
        if let Some(url) = &tool.download_url {
            println!("Would run: git clone {} && cd {} && cargo install --path .", 
                url, tool.name);
        }
        
        // Create tool cache directory
        fs::create_dir_all(&self.tool_cache)?;
        
        // Mark as installed (simulated)
        let install_marker = self.tool_cache.join(format!("{}.github-installed", tool.name));
        fs::write(install_marker, "github_installed")?;
        
        Ok(())
    }
    
    /// Verify local tool
    fn verify_local_tool(&self, tool: &CargoTool) -> Result<()> {
        println!("Verifying local tool: {}...", tool.name);
        
        // Check if tool is executable
        let tool_path = if tool.name.starts_with("cargo-") {
            format!("./target/release/{}", tool.name)
        } else {
            format!("./target/release/cargo-{}", tool.name)
        };
        
        if Path::new(&tool_path).exists() {
            println!("✓ Tool {} is available locally", tool.name);
        } else {
            println!("⚠ Tool {} not found at expected path", tool.name);
        }
        
        Ok(())
    }
    
    /// Install custom tool
    fn install_custom_tool(&self, tool: &CargoTool) -> Result<()> {
        println!("Installing custom tool: {}...", tool.name);
        
        // Custom installation would depend on tool requirements
        println!("Custom installation process for {}", tool.name);
        
        // Mark as installed
        let install_marker = self.tool_cache.join(format!("{}.custom-installed", tool.name));
        fs::write(install_marker, "custom_installed")?;
        
        Ok(())
    }
    
    /// Get list of integrated tools
    pub fn get_integrated_tools(&self) -> Vec<&CargoTool> {
        self.integrated_tools.values().collect()
    }
    
    /// Generate tool catalog
    pub fn generate_tool_catalog(&self, output_path: &Path) -> Result<()> {
        println!("Generating tool catalog...");
        
        let mut catalog = String::new();
        
        catalog.push_str("# Cargo Tool Catalog\n\n");
        catalog.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Integrated tools
        catalog.push_str("## Integrated Tools\n\n");
        catalog.push_str("| Name | Version | Source | Compatibility | Status |\n");
        catalog.push_str("|------|---------|--------|---------------|--------|\n");
        
        for tool in self.get_integrated_tools() {
            catalog.push_str(&format!("| {} | {} | {} | {:.1}% | {} |\n",
                tool.name,
                tool.version,
                self.format_source(&tool.source),
                tool.compatibility.compatibility_score * 100.0,
                self.format_integration_status(&tool.compatibility.integration_status)));
        }
        
        // Available tools (not integrated)
        catalog.push_str("\n## Available Tools (Not Integrated)\n\n");
        catalog.push_str("These tools were discovered but not yet integrated:\n\n");
        
        // Search for available tools
        let query = ToolSearchQuery {
            keywords: vec!["analysis".to_string(), "performance".to_string()],
            categories: vec!["analysis".to_string()],
            min_downloads: None,
            rust_version: None,
            platform: None,
            limit: 20,
        };
        
        let available_tools = self.search_all_sources(&query)?;
        let available_tools: Vec<&CargoTool> = available_tools.iter()
            .filter(|t| !self.integrated_tools.contains_key(&t.name))
            .collect();
        
        if !available_tools.is_empty() {
            catalog.push_str("| Name | Version | Source | Description |\n");
            catalog.push_str("|------|---------|--------|-------------|\n");
            
            for tool in available_tools.iter().take(10) {
                catalog.push_str(&format!("| {} | {} | {} | {} |\n",
                    tool.name, tool.version, self.format_source(&tool.source), tool.description));
            }
        } else {
            catalog.push_str("No additional tools found.\n");
        }
        
        // Tool sources summary
        catalog.push_str("\n## Tool Sources Summary\n\n");
        catalog.push_str(&format!("- **Integrated Tools**: {}\n", self.integrated_tools.len()));
        catalog.push_str(&format!("- **Available Tools**: {}\n", available_tools.len()));
        catalog.push_str("- **Search Sources**: crates.io, GitHub, local installation\n");
        
        fs::write(output_path, catalog)
            .context("Failed to write tool catalog")?;
        
        println!("Tool catalog generated at {}", output_path.display());
        Ok(())
    }
    
    /// Generate tool integration report
    pub fn generate_integration_report(&self, tool: &CargoTool, output_path: &Path) -> Result<()> {
        let mut report = String::new();
        
        report.push_str("# Tool Integration Report\n\n");
        report.push_str(&format!("Tool: {}\n", tool.name));
        report.push_str(&format!("Version: {}\n", tool.version));
        report.push_str(&format!("Source: {}\n\n", self.format_source(&tool.source)));
        
        // Description
        report.push_str(&format!("## Description\n\n{}\n\n", tool.description));
        
        // Compatibility analysis
        report.push_str("## Compatibility Analysis\n\n");
        report.push_str(&format!("- **Compatibility Score**: {:.1}%\n", 
            tool.compatibility.compatibility_score * 100.0));
        report.push_str(&format!("- **Integration Status**: {}\n", 
            self.format_integration_status(&tool.compatibility.integration_status)));
        report.push_str(&format!("- **Rust Version**: {}\n", 
            tool.compatibility.rust_version.as_deref().unwrap_or("unknown")));
        report.push_str(&format!("- **Platform Support**: {}\n", 
            tool.compatibility.platform_support.join(", ")));
        
        if !tool.compatibility.dependency_conflicts.is_empty() {
            report.push_str("\n### Dependency Conflicts\n\n");
            for conflict in &tool.compatibility.dependency_conflicts {
                report.push_str(&format!("- {}\n", conflict));
            }
        } else {
            report.push_str("\n### Dependency Conflicts\n\nNone detected\n");
        }
        
        // Integration steps
        report.push_str("\n## Integration Steps\n\n");
        
        match tool.source {
            ToolSource::CratesIo => {
                report.push_str(&format!("1. Run: `cargo install {}`\n", tool.name));
                report.push_str("2. Verify installation: `cargo <tool> --version`\n");
                report.push_str("3. Configure integration in cargo-vendormod\n");
            }
            ToolSource::GitHub => {
                if let Some(url) = &tool.download_url {
                    report.push_str(&format!("1. Clone repository: `git clone {}`\n", url));
                    report.push_str(&format!("2. Build and install: `cd {} && cargo install --path .`\n", 
                        tool.name));
                }
                report.push_str("3. Verify installation\n");
                report.push_str("4. Configure integration\n");
            }
            ToolSource::Local => {
                report.push_str("1. Tool is already available locally\n");
                report.push_str("2. Verify executable path\n");
                report.push_str("3. Configure integration\n");
            }
            ToolSource::Custom => {
                report.push_str("1. Follow custom installation instructions\n");
                report.push_str("2. Verify tool functionality\n");
                report.push_str("3. Configure integration\n");
            }
        }
        
        // Expected benefits
        report.push_str("\n## Expected Benefits\n\n");
        report.push_str("- Enhanced performance analysis\n");
        report.push_str("- Additional metrics and insights\n");
        report.push_str("- Improved correlation with other tools\n");
        report.push_str("- Better workload optimization\n");
        
        fs::write(output_path, report)
            .context("Failed to write integration report")?;
        
        Ok(())
    }
    
    /// Helper to format source
    fn format_source(&self, source: &ToolSource) -> &str {
        match source {
            ToolSource::CratesIo => "crates.io",
            ToolSource::GitHub => "GitHub",
            ToolSource::Local => "Local",
            ToolSource::Custom => "Custom",
        }
    }
    
    /// Helper to format integration status
    fn format_integration_status(&self, status: &IntegrationStatus) -> &str {
        match status {
            IntegrationStatus::NotIntegrated => "Not Integrated",
            IntegrationStatus::Partial => "Partial",
            IntegrationStatus::Full => "Full",
            IntegrationStatus::Native => "Native",
        }
    }
    
    /// Export tool information to JSON
    pub fn export_tool_info(&self, tools: &[CargoTool], output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(tools)
            .context("Failed to serialize tool information")?;
        
        fs::write(output_path, json)
            .context("Failed to write tool information")?;
        
        Ok(())
    }
    
    /// Import tool information from JSON
    pub fn import_tool_info(&self, input_path: &Path) -> Result<Vec<CargoTool>> {
        let json = fs::read_to_string(input_path)
            .context("Failed to read tool information")?;
        
        let tools: Vec<CargoTool> = serde_json::from_str(&json)
            .context("Failed to deserialize tool information")?;
        
        Ok(tools)
    }
}