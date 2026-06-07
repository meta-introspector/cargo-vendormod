//! # Configuration Module
//!
//! Provides configuration management for cargo-vendormod with support for:
//! - Config file loading (./vendormod.toml, ~/.config/cargo-vendormod/config.toml)
//! - Environment variables (VENDORMOD_* prefix)
//! - CLI argument overrides
//! - Sensible defaults

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Git executable path
    pub git_path: PathBuf,
    /// Default vendor directory
    pub vendor_dir: PathBuf,
    /// Default submodules directory
    pub submodules_dir: PathBuf,
    /// Default bare mirrors directory
    pub mirrors_dir: PathBuf,
    /// Default manifest path
    pub manifest_path: Option<PathBuf>,
    /// Default target branch
    pub target_branch: String,
    /// Version branch format (use {} as placeholder)
    pub version_branch_format: String,
    /// Whether to create version branches automatically
    pub create_version_branches: bool,
    /// Default number of threads for parallel operations
    pub default_threads: usize,
    /// Config file path (if explicitly provided)
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
    /// Directories to warm the cache with
    #[serde(default)]
    pub warm_dirs: Vec<WarmDir>,
}

/// A directory to be warmed in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmDir {
    pub path: PathBuf,
    pub prefix: String,
    #[serde(default)]
    pub exts: Vec<String>,
    #[serde(default)]
    pub max_size: u64,
    #[serde(default)]
    pub last_warmed: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let git_path = std::process::Command::new("which")
            .arg("git")
            .output()
            .ok()
            .and_then(|o| o.status.success().then_some(o))
            .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()))
            .unwrap_or_else(|| PathBuf::from("git"));

        Self {
            git_path,
            vendor_dir: PathBuf::from("vendor"),
            submodules_dir: PathBuf::from("submodules"),
            mirrors_dir: home.join("git"),
            manifest_path: None,
            target_branch: String::from("main"),
            version_branch_format: String::from("v{}"),
            create_version_branches: false,
            default_threads: num_cpus::get(),
            config_file: None,
            warm_dirs: Vec::new(),
        }
    }
}

impl Config {
    /// Load configuration from file with environment variable overrides
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let mut config = Config::default();

        // Try to load from explicit path, then default locations
        if let Some(path) = config_path {
            config.merge_file(path)?;
            config.config_file = Some(path.to_path_buf());
        } else {
            // Check for local config file
            let local_path = PathBuf::from("./vendormod.toml");
            if local_path.exists() {
                config.merge_file(&local_path)?;
                config.config_file = Some(local_path);
            } else if let Some(home) = dirs::config_dir() {
                let global_config = home.join("cargo-vendormod").join("config.toml");
                if global_config.exists() {
                    config.merge_file(&global_config)?;
                    config.config_file = Some(global_config);
                }
            }
        }

        // Override with environment variables
        config.merge_env();

        Ok(config)
    }

    fn merge_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let parsed: ConfigFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // Apply values from config file
        if let Some(p) = parsed.git_path {
            self.git_path = p;
        }
        if let Some(p) = parsed.vendor_dir {
            self.vendor_dir = p;
        }
        if let Some(p) = parsed.submodules_dir {
            self.submodules_dir = p;
        }
        if let Some(p) = parsed.mirrors_dir {
            self.mirrors_dir = p;
        }
        if let Some(p) = parsed.manifest_path {
            self.manifest_path = Some(p);
        }
        if let Some(b) = parsed.target_branch {
            self.target_branch = b;
        }
        if let Some(f) = parsed.version_branch_format {
            self.version_branch_format = f;
        }
        if let Some(b) = parsed.create_version_branches {
            self.create_version_branches = b;
        }
        if let Some(t) = parsed.default_threads {
            self.default_threads = t;
        }
        if let Some(d) = parsed.warm_dirs {
            self.warm_dirs = d;
        }

        Ok(())
    }

    fn merge_env(&mut self) {
        // VENDORMOD_GIT_PATH
        if let Ok(p) = std::env::var("VENDORMOD_GIT_PATH") {
            self.git_path = PathBuf::from(p);
        }
        // VENDORMOD_VENDOR_DIR
        if let Ok(p) = std::env::var("VENDORMOD_VENDOR_DIR") {
            self.vendor_dir = PathBuf::from(p);
        }
        // VENDORMOD_SUBMODULES_DIR
        if let Ok(p) = std::env::var("VENDORMOD_SUBMODULES_DIR") {
            self.submodules_dir = PathBuf::from(p);
        }
        // VENDORMOD_MIRRORS_DIR
        if let Ok(p) = std::env::var("VENDORMOD_MIRRORS_DIR") {
            self.mirrors_dir = PathBuf::from(p);
        }
        // VENDORMOD_TARGET_BRANCH
        if let Ok(b) = std::env::var("VENDORMOD_TARGET_BRANCH") {
            self.target_branch = b;
        }
        // VENDORMOD_CREATE_VERSION_BRANCHES
        if let Ok(b) = std::env::var("VENDORMOD_CREATE_VERSION_BRANCHES") {
            self.create_version_branches = b.parse().unwrap_or(false);
        }
        // VENDORMOD_THREADS
        if let Ok(t) = std::env::var("VENDORMOD_THREADS") {
            self.default_threads = t.parse().unwrap_or(self.default_threads);
        }
    }

    /// Get mirrors path for a specific owner/repo
    pub fn get_mirror_path(&self, owner: &str, repo: &str) -> PathBuf {
        self.mirrors_dir.join(owner).join(format!("{}.git", repo))
    }

    /// Get submodule path for a specific repo
    pub fn get_submodule_path(&self, repo_name: &str) -> PathBuf {
        self.submodules_dir.join(repo_name)
    }

    /// Generate version branch name
    pub fn version_branch(&self, version: &str) -> String {
        self.version_branch_format.replace("{}", version)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = self.config_file.clone().unwrap_or_else(|| PathBuf::from("./vendormod.toml"));
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }
}

/// Intermediate structure for config file parsing
#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    git_path: Option<PathBuf>,
    vendor_dir: Option<PathBuf>,
    submodules_dir: Option<PathBuf>,
    mirrors_dir: Option<PathBuf>,
    manifest_path: Option<PathBuf>,
    target_branch: Option<String>,
    version_branch_format: Option<String>,
    create_version_branches: Option<bool>,
    default_threads: Option<usize>,
    warm_dirs: Option<Vec<WarmDir>>,
}

/// Generate a sample config file
pub fn generate_sample_config() -> String {
    r#"# cargo-vendormod configuration
# Copy to ./vendormod.toml or ~/.config/cargo-vendormod/config.toml

# Git executable path
# git_path = "git"

# Default directories (relative to workspace)
# vendor_dir = "vendor"
# submodules_dir = "submodules"

# Bare git mirrors directory (absolute path recommended)
# Example: ~/git or /home/user/git
# mirrors_dir = "~/git"

# Default target branch for submodules
# target_branch = "main"

# Version branch format (use {} as placeholder for version)
# version_branch_format = "v{}"

# Whether to automatically create version branches
# create_version_branches = false

# Default number of threads for parallel operations
# default_threads = 8

# Directories to warm the cache with
# [[warm_dirs]]
# path = "/path/to/dir"
# prefix = "my-prefix"
# exts = ["rs", "toml", "md"]
# max_size = 1048576
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.git_path.as_os_str().is_empty());
        assert_eq!(config.target_branch, "main");
    }

    #[test]
    fn test_version_branch() {
        let config = Config::default();
        assert_eq!(config.version_branch("1.0.0"), "v1.0.0");
        
        let mut config = Config::default();
        config.version_branch_format = "{}".to_string();
        assert_eq!(config.version_branch("1.0.0"), "1.0.0");
    }
}