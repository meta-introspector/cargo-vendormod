use anyhow::{Context, Result};
use chrono;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupLock {
    pub dependencies: HashMap<String, DependencyInfo>,
    pub submodules: HashMap<String, SubmoduleInfo>,
    pub patches: HashMap<String, PatchInfo>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub source: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleInfo {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub branch: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInfo {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
}

impl RollupLock {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            submodules: HashMap::new(),
            patches: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load(root_dir: &Path) -> Result<Self> {
        let lock_path = root_dir.join("rollup.lock");
        
        if !lock_path.exists() {
            // Create new RollupLock if file doesn't exist
            let mut rollup_lock = Self::new();
            rollup_lock.metadata.insert("created".to_string(), 
                chrono::Utc::now().to_rfc3339());
            return Ok(rollup_lock);
        }

        let content = fs::read_to_string(&lock_path)
            .with_context(|| format!("Failed to read rollup.lock at {:?}", lock_path))?;
        
        let rollup_lock: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse rollup.lock at {:?}", lock_path))?;

        Ok(rollup_lock)
    }

    pub fn save(&self, root_dir: &Path) -> Result<()> {
        let lock_path = root_dir.join("rollup.lock");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize RollupLock")?;
        
        fs::write(&lock_path, content)
            .with_context(|| format!("Failed to write rollup.lock to {:?}", lock_path))?;

        Ok(())
    }

    pub fn add_submodule(&mut self, name: String, path: PathBuf, url: String, branch: String) {
        let submodule_info = SubmoduleInfo {
            name: name.clone(),
            path,
            url,
            branch,
            commit: String::new(), // Will be updated after git operations
        };
        self.submodules.insert(name, submodule_info);
    }

    pub fn update_submodule_commit(&mut self, name: &str, commit: String) -> Result<()> {
        if let Some(submodule) = self.submodules.get_mut(name) {
            submodule.commit = commit;
            Ok(())
        } else {
            anyhow::bail!("Submodule {} not found in RollupLock", name)
        }
    }

    pub fn add_dependency(&mut self, name: String, version: String, source: String, path: Option<PathBuf>) {
        let dep_info = DependencyInfo {
            name: name.clone(),
            version,
            source,
            path,
        };
        self.dependencies.insert(name, dep_info);
    }

    pub fn update_dependency_path(&mut self, name: &str, path: PathBuf) -> Result<()> {
        if let Some(dep) = self.dependencies.get_mut(name) {
            dep.path = Some(path);
            Ok(())
        } else {
            anyhow::bail!("Dependency {} not found in RollupLock", name)
        }
    }

    pub fn add_patch(&mut self, name: String, path: PathBuf, description: String) {
        let patch_info = PatchInfo {
            name: name.clone(),
            path,
            description,
        };
        self.patches.insert(name, patch_info);
    }
}

impl Default for RollupLock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_lock() -> RollupLock {
        let mut lock = RollupLock::new();
        lock.dependencies.insert(
            "serde".to_string(),
            DependencyInfo {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: "crates.io".to_string(),
                path: None,
            },
        );
        lock.submodules.insert(
            "cargo-vendormod".to_string(),
            SubmoduleInfo {
                name: "cargo-vendormod".to_string(),
                path: PathBuf::from("submodules/cargo-vendormod"),
                url: "https://github.com/user/cargo-vendormod".to_string(),
                branch: "main".to_string(),
                commit: "abc123".to_string(),
            },
        );
        lock
    }

    #[test]
    fn test_rollup_lock_new() {
        let lock = RollupLock::new();
        assert!(lock.dependencies.is_empty());
        assert!(lock.submodules.is_empty());
        assert!(lock.patches.is_empty());
        assert!(lock.metadata.is_empty());
    }

    #[test]
    fn test_rollup_lock_default() {
        let lock = RollupLock::default();
        assert!(lock.dependencies.is_empty());
        assert!(lock.submodules.is_empty());
    }

    #[test]
    fn test_dependency_info_serialization() {
        let dep = DependencyInfo {
            name: "test-dep".to_string(),
            version: "0.1.0".to_string(),
            source: "crates.io".to_string(),
            path: Some(PathBuf::from("vendor/test-dep")),
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("test-dep"));
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn test_submodule_info_serialization() {
        let sub = SubmoduleInfo {
            name: "test-submodule".to_string(),
            path: PathBuf::from("submodules/test"),
            url: "https://github.com/test/repo".to_string(),
            branch: "main".to_string(),
            commit: "def456".to_string(),
        };
        let json = serde_json::to_string(&sub).unwrap();
        assert!(json.contains("test-submodule"));
        assert!(json.contains("def456"));
    }

    #[test]
    fn test_patch_info_creation() {
        let patch = PatchInfo {
            name: "security-patch".to_string(),
            path: PathBuf::from("patches/security.patch"),
            description: "Fix security vulnerability".to_string(),
        };
        assert_eq!(patch.name, "security-patch");
        assert_eq!(patch.description, "Fix security vulnerability");
    }

    #[test]
    fn test_add_submodule() {
        let mut lock = RollupLock::new();
        lock.add_submodule(
            "new-submodule".to_string(),
            PathBuf::from("submodules/new"),
            "https://github.com/user/new".to_string(),
            "develop".to_string(),
        );
        assert!(lock.submodules.contains_key("new-submodule"));
        let sub = lock.submodules.get("new-submodule").unwrap();
        assert_eq!(sub.branch, "develop");
        assert!(sub.commit.is_empty());
    }

    #[test]
    fn test_update_submodule_commit() {
        let mut lock = RollupLock::new();
        lock.add_submodule(
            "test-mod".to_string(),
            PathBuf::from("submodules/test"),
            "https://github.com/test".to_string(),
            "main".to_string(),
        );
        assert!(lock.update_submodule_commit("test-mod", "newcommit123".to_string()).is_ok());
        assert_eq!(lock.submodules.get("test-mod").unwrap().commit, "newcommit123");
    }

    #[test]
    fn test_update_submodule_commit_not_found() {
        let mut lock = RollupLock::new();
        let result = lock.update_submodule_commit("nonexistent", "commit".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_dependency() {
        let mut lock = RollupLock::new();
        lock.add_dependency(
            "tokio".to_string(),
            "1.0.0".to_string(),
            "crates.io".to_string(),
            Some(PathBuf::from("vendor/tokio")),
        );
        assert!(lock.dependencies.contains_key("tokio"));
        let dep = lock.dependencies.get("tokio").unwrap();
        assert_eq!(dep.version, "1.0.0");
        assert!(dep.path.is_some());
    }

    #[test]
    fn test_update_dependency_path() {
        let mut lock = RollupLock::new();
        lock.add_dependency(
            "async-std".to_string(),
            "1.0.0".to_string(),
            "crates.io".to_string(),
            None,
        );
        assert!(lock.update_dependency_path("async-std", PathBuf::from("new/path")).is_ok());
        assert_eq!(lock.dependencies.get("async-std").unwrap().path, Some(PathBuf::from("new/path")));
    }

    #[test]
    fn test_update_dependency_path_not_found() {
        let mut lock = RollupLock::new();
        let result = lock.update_dependency_path("nonexistent", PathBuf::from("path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_patch() {
        let mut lock = RollupLock::new();
        lock.add_patch(
            "fix-bug".to_string(),
            PathBuf::from("patches/fix.patch"),
            "Fixes issue #123".to_string(),
        );
        assert!(lock.patches.contains_key("fix-bug"));
    }

    #[test]
    fn test_rollup_lock_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let result = RollupLock::load(temp_dir.path());
        assert!(result.is_ok());
        let lock = result.unwrap();
        // Nonexistent lock file should create a new lock with created metadata
        assert!(!lock.metadata.is_empty());
    }

    #[test]
    fn test_rollup_lock_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("rollup.lock");
        
        let mut lock = RollupLock::new();
        lock.add_submodule(
            "saved-module".to_string(),
            PathBuf::from("submodules/saved"),
            "https://github.com/test/saved".to_string(),
            "main".to_string(),
        );
        
        lock.save(temp_dir.path()).unwrap();
        assert!(lock_path.exists());
        
        let loaded = RollupLock::load(temp_dir.path()).unwrap();
        assert!(loaded.submodules.contains_key("saved-module"));
    }
}