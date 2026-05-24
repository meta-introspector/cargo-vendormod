use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

lazy_static! {
    static ref GITHUB_URL_RE: Regex =
        Regex::new(r"github\.com/([^/]+)/([^/.]+)(?:/tree/[^/]+/.+)?(?:\.git)?").unwrap();
}

const DEFAULT_MIRRORS_PATH: &str = "/home/mdupont/git/host";

/// Represents a package entry from a Cargo.lock file
#[derive(Clone, Debug)]
pub struct LockfilePackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub commit: Option<String>,
    pub is_workspace_member: bool,
    pub bare_repo_path: Option<String>,
    pub github_owner: String,
    pub github_repo: String,
}

impl LockfilePackage {
    /// Check if this package is from a git repository
    pub fn is_git(&self) -> bool {
        self.source.starts_with("git+")
    }

    /// Check if this package is from the crates.io registry
    pub fn is_registry(&self) -> bool {
        !self.is_git() && !self.is_path()
    }

    /// Check if this package is from a local path
    pub fn is_path(&self) -> bool {
        self.source.contains("path+")
            || (self.source.contains("path")
                && !self.source.contains("git+")
                && !self.source.is_empty())
            || (!self.is_git() && self.source.is_empty())
    }

    /// Generate the GitHub URL for this package
    pub fn github_url(&self) -> String {
        if !self.github_owner.is_empty() && !self.github_repo.is_empty() {
            format!(
                "https://github.com/{}/{}",
                self.github_owner, self.github_repo
            )
        } else {
            String::new()
        }
    }
}

/// Parse owner and repo name from a GitHub URL
fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Handle git@github.com:owner/repo format first (must come before generic github.com check)
    if url.contains("@github.com:") {
        let parts: Vec<&str> = url.split("@github.com:").collect();
        if parts.len() == 2 {
            let path_parts: Vec<&str> = parts[1].split('/').collect();
            if path_parts.len() >= 2 {
                let repo_part = path_parts[1].split('.').next().unwrap_or(path_parts[1]);
                return Some((path_parts[0].to_string(), repo_part.to_string()));
            }
        }
    }
    // Handle github.com/owner/repo format
    if url.contains("github.com/") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let path_parts: Vec<&str> = parts[1].split('/').collect();
            if path_parts.len() >= 2 {
                let repo_part = path_parts[1].split('.').next().unwrap_or(path_parts[1]);
                return Some((path_parts[0].to_string(), repo_part.to_string()));
            }
        }
    }
    None
}

/// Compute the bare repository path for a GitHub package
fn compute_bare_repo_path(owner: &str, repo: &str, mirrors_path: &Path) -> String {
    mirrors_path
        .join(owner)
        .join(format!("{}.git", repo))
        .to_string_lossy()
        .to_string()
}

/// Find all workspace member Cargo.toml files
///
/// Returns the root Cargo.toml and any explicitly defined workspace member Cargo.toml files.
pub fn find_workspace_members(workspace_path: &Path) -> Result<Vec<PathBuf>> {
    let mut members = Vec::new();
    let cargo_toml_path = workspace_path.join("Cargo.toml");

    if cargo_toml_path.exists() {
        members.push(cargo_toml_path.clone());

        // Read the workspace manifest to find member patterns
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read Cargo.toml at {:?}", cargo_toml_path))?;

        // Try to parse as toml to find workspace members
        if let Ok(doc) = content.parse::<toml_edit::DocumentMut>() {
            if let Some(workspace) = doc.get("workspace") {
                if let Some(members_list) = workspace.get("members") {
                    if let Some(array) = members_list.as_array() {
                        for pattern in array.iter().filter_map(|p| p.as_str()) {
                            let search_pattern = workspace_path.join(pattern).join("Cargo.toml");
                            if let Ok(paths) = glob::glob(&search_pattern.to_string_lossy()) {
                                for path in paths.filter_map(|p| p.ok()) {
                                    if !members.contains(&path) {
                                        members.push(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(members)
}

/// Parse a single Cargo.lock file and extract package information
fn parse_lockfile(lock_path: &Path) -> Result<Vec<LockfilePackage>> {
    let lock_content = fs::read_to_string(lock_path)
        .with_context(|| format!("Failed to read Cargo.lock at {:?}", lock_path))?;

    let lock_doc = lock_content
        .parse::<DocumentMut>()
        .context("Failed to parse Cargo.lock")?;

    let package_array = match lock_doc.get("package") {
        Some(toml_edit::Item::ArrayOfTables(arr)) => arr,
        _ => return Ok(Vec::new()),
    };

    let mirrors_path = Path::new(DEFAULT_MIRRORS_PATH);
    let mut packages = Vec::new();

    for table in package_array {
        let name = match table
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s: &str| s.to_string())
        {
            Some(n) => n,
            None => continue,
        };

        let version = table
            .get("version")
            .and_then(|v: &toml_edit::Item| v.as_str())
            .map(|s: &str| s.to_string())
            .unwrap_or_default();

        let source = table
            .get("source")
            .and_then(|v: &toml_edit::Item| v.as_str())
            .map(|s: &str| s.to_string())
            .unwrap_or_default();

        let commit = table
            .get("checksum")
            .and_then(|v: &toml_edit::Item| v.as_str())
            .map(|s: &str| s.to_string());

        // Parse git commit from source if available
        let commit = table
            .get("source")
            .and_then(|v: &toml_edit::Item| v.as_str())
            .and_then(|s: &str| {
                if s.contains("#") {
                    s.split('#').nth(1).map(|x: &str| x.to_string())
                } else {
                    None
                }
            })
            .or(commit);

        // Determine package type and extract GitHub info
        let (github_owner, github_repo, bare_repo_path) = if source.starts_with("git+") {
            // Strip git+ prefix and query params
            let base = source.split('#').next().unwrap_or(&source);
            let cleaned = base.trim_start_matches("git+");
            let cleaned = cleaned.split('?').next().unwrap_or(cleaned);

            if let Some((owner, repo)) = parse_github_url(cleaned) {
                let bare_repo = compute_bare_repo_path(&owner, &repo, mirrors_path);
                (owner, repo, Some(bare_repo))
            } else if let Some(caps) = GITHUB_URL_RE.captures(cleaned) {
                let owner = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let repo = caps.get(2).map_or("", |m| m.as_str()).to_string();
                let bare_repo = compute_bare_repo_path(&owner, &repo, mirrors_path);
                (owner, repo, Some(bare_repo))
            } else {
                (String::new(), String::new(), None)
            }
        } else {
            (String::new(), String::new(), None)
        };

        packages.push(LockfilePackage {
            name,
            version,
            source,
            commit,
            is_workspace_member: false,
            bare_repo_path,
            github_owner,
            github_repo,
        });
    }

    Ok(packages)
}

/// Parse multiple Cargo.lock files in parallel
pub fn parse_lockfiles_parallel(lockfiles: &[PathBuf]) -> Result<Vec<LockfilePackage>> {
    let results: Vec<Vec<LockfilePackage>> = lockfiles
        .par_iter()
        .map(|lock_path| {
            parse_lockfile(lock_path)
                .with_context(|| format!("Failed to parse lockfile {:?}", lock_path))
        })
        .collect::<Result<Vec<Vec<LockfilePackage>>>>()?;

    // Deduplicate by name to avoid counting the same crate multiple times
    let mut unique_packages = std::collections::HashMap::new();
    for packages in results {
        for pkg in packages {
            unique_packages.entry(pkg.name.clone()).or_insert(pkg);
        }
    }

    let mut packages: Vec<LockfilePackage> = unique_packages.into_values().collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(packages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_git_at_format() {
        let result = parse_github_url("git@github.com:rust-lang/cargo");
        assert_eq!(result, Some(("rust-lang".to_string(), "cargo".to_string())));
    }

    #[test]
    fn test_parse_github_url_https_format() {
        let result = parse_github_url("https://github.com/rust-lang/cargo");
        assert_eq!(result, Some(("rust-lang".to_string(), "cargo".to_string())));
    }

    #[test]
    fn test_parse_github_url_with_git_extension() {
        let result = parse_github_url("https://github.com/rust-lang/cargo.git");
        assert_eq!(result, Some(("rust-lang".to_string(), "cargo".to_string())));
    }

    #[test]
    fn test_is_git() {
        let pkg = LockfilePackage {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            source: "git+https://github.com/foo/bar".to_string(),
            commit: None,
            is_workspace_member: false,
            bare_repo_path: None,
            github_owner: String::new(),
            github_repo: String::new(),
        };
        assert!(pkg.is_git());
    }

    #[test]
    fn test_is_registry() {
        let pkg = LockfilePackage {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            source: "registry+crates-io".to_string(),
            commit: None,
            is_workspace_member: false,
            bare_repo_path: None,
            github_owner: String::new(),
            github_repo: String::new(),
        };
        assert!(pkg.is_registry());
    }

    #[test]
    fn test_compute_bare_repo_path() {
        let path = compute_bare_repo_path("rust-lang", "cargo", Path::new("/home/user/git"));
        assert_eq!(path, "/home/user/git/rust-lang/cargo.git");
    }
}
