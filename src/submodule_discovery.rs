use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::DocumentMut;

const MAX_SUBMODULE_DEPTH: usize = 8;

#[derive(Debug, Clone)]
pub struct SubmoduleInfo {
    pub path: PathBuf,
    pub url: String,
    pub branch: Option<String>,
}

pub fn discover_submodules(repo_path: &Path) -> Result<Vec<SubmoduleInfo>> {
    let submodules = discover_via_git_submodule_status(repo_path)?;
    if !submodules.is_empty() {
        return Ok(submodules);
    }

    let gitmodules_path = repo_path.join(".gitmodules");
    if gitmodules_path.exists() {
        let mut all_submodules = Vec::new();
        discover_recursive(repo_path, 0, &mut all_submodules)?;
        return Ok(all_submodules);
    }

    Ok(Vec::new())
}

fn discover_via_git_submodule_status(repo_path: &Path) -> Result<Vec<SubmoduleInfo>> {
    let output = Command::new("git")
        .args(["submodule", "status"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git submodule status")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut submodules = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let path = parts[1];
            let path_buf = PathBuf::from(path);

            let url = get_submodule_url_from_config(repo_path, path);

            submodules.push(SubmoduleInfo {
                path: path_buf,
                url,
                branch: None,
            });
        }
    }

    Ok(submodules)
}

fn get_submodule_url_from_config(repo_path: &Path, submodule_path: &str) -> String {
    let output = Command::new("git")
        .args(["config", "--get", &format!("submodule.{}.url", submodule_path)])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}

fn discover_recursive(base_path: &Path, depth: usize, submodules: &mut Vec<SubmoduleInfo>) -> Result<()> {
    if depth >= MAX_SUBMODULE_DEPTH {
        return Ok(());
    }

    let gitmodules_path = base_path.join(".gitmodules");
    if !gitmodules_path.exists() {
        return Ok(());
    }

    let parsed = parse_gitmodules(&gitmodules_path)?;
    for sm in parsed {
        let full_path = base_path.join(&sm.path);
        submodules.push(SubmoduleInfo {
            path: full_path.clone(),
            url: sm.url.clone(),
            branch: sm.branch.clone(),
        });

        if full_path.exists() && full_path.is_dir() {
            let _ = discover_recursive(&full_path, depth + 1, submodules);
        }
    }

    Ok(())
}

fn parse_gitmodules(gitmodules_path: &Path) -> Result<Vec<SubmoduleInfo>> {
    let content = fs::read_to_string(gitmodules_path)
        .context("Failed to read .gitmodules")?;

    let doc: DocumentMut = content.parse()
        .context("Failed to parse .gitmodules")?;

    let mut submodules = Vec::new();
    let mut current_section = String::new();

    for (key, item) in doc.iter() {
        if key.starts_with("submodule ") {
            current_section = key.trim_start_matches("submodule ").trim_matches('"').to_string();
        }

        if current_section.is_empty() {
            continue;
        }

        match key {
            k if k.contains(".path") => {
                if let Some(path_val) = item.as_str() {
                    let url = get_submodule_url(&doc, &current_section);
                    let branch = get_submodule_branch(&doc, &current_section);
                    submodules.push(SubmoduleInfo {
                        path: PathBuf::from(path_val),
                        url: url.unwrap_or_default(),
                        branch,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(submodules)
}

fn get_submodule_url(doc: &DocumentMut, section: &str) -> Option<String> {
    let key = format!("submodule \"{}\".url", section);
    doc.get(&key).and_then(|v| v.as_str()).map(String::from)
}

fn get_submodule_branch(doc: &DocumentMut, section: &str) -> Option<String> {
    let key = format!("submodule \"{}\".branch", section);
    doc.get(&key).and_then(|v| v.as_str()).map(String::from)
}

pub fn normalize_url_to_flat_path(url: &str) -> PathBuf {
    let url = url.trim();

    let cleaned = if url.starts_with("git@") {
        url.trim_start_matches("git@")
            .replace(":", "/")
    } else if url.starts_with("https://") {
        url.trim_start_matches("https://").to_string()
    } else if url.starts_with("http://") {
        url.trim_start_matches("http://").to_string()
    } else {
        url.to_string()
    };

    let path = cleaned
        .trim_end_matches(".git")
        .trim_end_matches("/");

    PathBuf::from(path)
}

pub fn clone_submodules_to_target(
    submodules: Vec<SubmoduleInfo>,
    target_base: &Path,
    git_exe: &Path,
) -> Result<()> {
    fs::create_dir_all(target_base).context("Failed to create target directory")?;

    for sm in submodules {
        let flat_path = normalize_url_to_flat_path(&sm.url);
        let target_path = target_base.join(&flat_path);

        if target_path.exists() {
            println!("Submodule already exists at {}, skipping", target_path.display());
            continue;
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).context(format!("Failed to create parent for {}", parent.display()))?;
        }

        println!("Cloning submodule: {} -> {}", sm.url, target_path.display());

        let output = Command::new(git_exe)
            .arg("clone")
            .arg(&sm.url)
            .arg(&target_path)
            .output()
            .context(format!("Failed to clone submodule from {}", sm.url))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: Failed to clone {}: {}", sm.url, err);
            continue;
        }

        if let Some(branch) = &sm.branch {
            let _ = Command::new(git_exe)
                .arg("-C")
                .arg(&target_path)
                .arg("checkout")
                .arg(branch)
                .output();
        }

        println!("Cloned submodule: {}", sm.url);
    }

    Ok(())
}

pub fn get_git_executable() -> PathBuf {
    PathBuf::from("git")
}