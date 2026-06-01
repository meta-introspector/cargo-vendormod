use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;
use regex::Regex;
use lazy_static::lazy_static;

use crate::context::AppContext;

lazy_static! {
    static ref GITHUB_URL_RE: Regex =
        Regex::new(r"github\.com/([^/]+)/([^/.]+)(?:/tree/[^/]+/.+)?(\.git)?").unwrap();
}

#[derive(Clone, Debug)]
pub struct RepoIdentifier {
    pub url: String,
    pub version: String,
    pub commit: Option<String>,
    pub owner: String,
    pub repo_name: String,
}

/// Parse Cargo.lock to extract git-based dependencies with their versions and commits.
pub fn collect_repo_info(ctx: &AppContext) -> Result<HashMap<String, RepoIdentifier>> {
    let lock_path = if ctx.manifest_path.with_extension("lock").exists() {
        ctx.manifest_path.with_extension("lock")
    } else {
        ctx.root_dir.join("Cargo.lock")
    };

    if !lock_path.exists() {
        anyhow::bail!("Cargo.lock not found at {:?}. Run `cargo generate-lockfile` first.", lock_path);
    }

    collect_repo_info_from_lock(&lock_path)
}

pub fn collect_repo_info_from_lock(lock_path: &Path) -> Result<HashMap<String, RepoIdentifier>> {
    if !lock_path.exists() {
        return Ok(HashMap::new());
    }

    let lock_content = fs::read_to_string(lock_path)
        .with_context(|| format!("Failed to read Cargo.lock at {:?}", lock_path))?;

    let lock_doc = lock_content
        .parse::<DocumentMut>()
        .context("Failed to parse Cargo.lock")?;

    let package_array = match lock_doc.get("package") {
        Some(toml_edit::Item::Value(toml_edit::Value::Array(arr))) => arr,
        _ => return Ok(HashMap::new()),
    };

    let mut repo_map: HashMap<String, RepoIdentifier> = HashMap::new();

    for pkg_value in package_array {
        let table = match pkg_value {
            toml_edit::Value::InlineTable(t) => t,
            _ => continue,
        };

        let _name = table.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Package missing name in Cargo.lock"))?;

        let version = table.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let source_opt = table.get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let source_str = match source_opt {
            Some(s) => s,
            None => continue,
        };

        if !source_str.starts_with("git+") {
            continue;
        }

        let base = source_str.split('#').next().unwrap_or(&source_str);
        let cleaned = base.trim_start_matches("git+");
        let cleaned = cleaned.split('?').next().unwrap_or(cleaned).to_string();

        let commit = source_str.split('#').nth(1).map(|s| s.to_string());

        let owner_repo_capture = GITHUB_URL_RE.captures(&cleaned);
        let (owner, repo_name) = match owner_repo_capture {
            Some(caps) => (
                caps.get(1).map_or("", |m| m.as_str()).to_string(),
                caps.get(2).map_or("", |m| m.as_str()).to_string(),
            ),
            None => continue,
        };

        if repo_name.is_empty() {
            continue;
        }

        let identifier = RepoIdentifier {
            url: cleaned.clone(),
            version,
            commit: commit.clone(),
            owner,
            repo_name: repo_name.clone(),
        };

        repo_map.insert(repo_name, identifier);
    }

    Ok(repo_map)
}


fn parse_github_url_from_url(url: &str) -> Option<(String, String)> {
    if url.contains("@github.com:") {
        let parts: Vec<&str> = url.split("@github.com:").collect();
        if parts.len() == 2 {
            let path_parts: Vec<&str> = parts[1].split('/').collect();
            if path_parts.len() >= 2 {
                let rp = path_parts[1].trim_end_matches(".git");
                return Some((path_parts[0].to_string(), rp.to_string()));
            }
        }
    }
    if url.contains("github.com/") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let path_parts: Vec<&str> = parts[1].split('/').collect();
            if path_parts.len() >= 2 {
                let repo = path_parts[1].trim_end_matches(".git");
                return Some((path_parts[0].to_string(), repo.to_string()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_identifier_creation() {
        let identifier = RepoIdentifier {
            url: "https://github.com/rust-lang/cargo".to_string(),
            version: "1.0.0".to_string(),
            commit: Some("abc123".to_string()),
            owner: "rust-lang".to_string(),
            repo_name: "cargo".to_string(),
        };
        assert_eq!(identifier.url, "https://github.com/rust-lang/cargo");
        assert_eq!(identifier.version, "1.0.0");
        assert_eq!(identifier.commit, Some("abc123".to_string()));
        assert_eq!(identifier.owner, "rust-lang");
        assert_eq!(identifier.repo_name, "cargo");
    }

    #[test]
    fn test_repo_identifier_without_commit() {
        let identifier = RepoIdentifier {
            url: "https://github.com/rust-lang/rust".to_string(),
            version: "1.70.0".to_string(),
            commit: None,
            owner: "rust-lang".to_string(),
            repo_name: "rust".to_string(),
        };
        assert!(identifier.commit.is_none());
    }

    #[test]
    fn test_parse_github_url_from_url_ssh_style() {
        let result = parse_github_url_from_url("git@github.com:rust-lang/cargo.git");
        assert_eq!(result, Some(("rust-lang".to_string(), "cargo".to_string())));
    }

    #[test]
    fn test_parse_github_url_from_url_https_style() {
        let result = parse_github_url_from_url("https://github.com/rust-lang/serde");
        assert_eq!(result, Some(("rust-lang".to_string(), "serde".to_string())));
    }

    #[test]
    fn test_parse_github_url_from_url_with_git_suffix() {
        let result = parse_github_url_from_url("git@github.com:owner/repo.git.git");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_from_url_invalid() {
        let result = parse_github_url_from_url("https://gitlab.com/owner/repo");
        assert!(result.is_none());
    }

    #[test]
    fn test_collect_repo_info_from_lock_empty() {
        // Test with non-existent lock file
        let result = collect_repo_info_from_lock(Path::new("/nonexistent/Cargo.lock"));
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty());
    }
}
