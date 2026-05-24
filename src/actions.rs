use std::path::PathBuf;
use std::collections::HashMap;

use crate::repo_collection::RepoIdentifier;

#[derive(Clone, Debug, serde::Serialize)]
pub struct RepoAction {
    pub repo_url: String,
    pub owner: String,
    pub repo_name: String,
    pub submodule_path: PathBuf,
    pub target_branch: String,
    pub mirrors_path: PathBuf,
    pub version: String,
    pub desired_commit: Option<String>,
}

pub fn create_actions_plan(ctx: &crate::context::AppContext, repo_info_map: &HashMap<String, RepoIdentifier>) -> Vec<RepoAction> {
    let submodules_dir = &ctx.submodules_dir;
    repo_info_map.iter()
        .filter_map(|(url, identifier)| {
            if identifier.owner.is_empty() || identifier.repo_name.is_empty() {
                eprintln!("Could not extract owner/repo from {}. Skipping.", url);
                return None;
            }
            let submodule_path = submodules_dir.join(&identifier.repo_name);
            Some(RepoAction {
                repo_url: identifier.url.clone(),
                owner: identifier.owner.clone(),
                repo_name: identifier.repo_name.clone(),
                submodule_path,
                target_branch: ctx.target_branch.clone(),
                mirrors_path: ctx.mirrors_path.clone(),
                version: identifier.version.clone(),
                desired_commit: identifier.commit.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AppContext;
    use std::path::PathBuf;

    #[test]
    fn test_repo_action_creation() {
        let action = RepoAction {
            repo_url: "https://github.com/rust-lang/cargo".to_string(),
            owner: "rust-lang".to_string(),
            repo_name: "cargo".to_string(),
            submodule_path: PathBuf::from("submodules/cargo"),
            target_branch: "main".to_string(),
            mirrors_path: PathBuf::from("/mirrors"),
            version: "1.0.0".to_string(),
            desired_commit: Some("abc123".to_string()),
        };
        assert_eq!(action.repo_name, "cargo");
        assert_eq!(action.version, "1.0.0");
        assert!(action.desired_commit.is_some());
    }

    #[test]
    fn test_create_actions_plan_empty() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = AppContext {
            git_executable_path: PathBuf::from("git"),
            root_dir: dir.path().to_path_buf(),
            manifest_path: dir.path().join("Cargo.toml"),
            submodules_dir: dir.path().join("submodules"),
            mirrors_path: PathBuf::from("/mirrors"),
            vendor_dir: dir.path().join("vendor"),
            target_branch: "main".to_string(),
            create_version_branches: false,
            version_branch_format: "v{}".to_string(),
            dry_run: false,
            verbose: false,
            temp_symlink: None,
        };
        let repo_info: HashMap<String, crate::repo_collection::RepoIdentifier> = HashMap::new();
        let actions = create_actions_plan(&ctx, &repo_info);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_create_actions_plan_with_items() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = AppContext {
            git_executable_path: PathBuf::from("git"),
            root_dir: dir.path().to_path_buf(),
            manifest_path: dir.path().join("Cargo.toml"),
            submodules_dir: dir.path().join("submodules"),
            mirrors_path: PathBuf::from("/mirrors"),
            vendor_dir: dir.path().join("vendor"),
            target_branch: "main".to_string(),
            create_version_branches: false,
            version_branch_format: "v{}".to_string(),
            dry_run: false,
            verbose: false,
            temp_symlink: None,
        };
        let mut repo_info: HashMap<String, crate::repo_collection::RepoIdentifier> = HashMap::new();
        repo_info.insert("cargo".to_string(), crate::repo_collection::RepoIdentifier {
            url: "https://github.com/rust-lang/cargo".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
            owner: "rust-lang".to_string(),
            repo_name: "cargo".to_string(),
        });
        let actions = create_actions_plan(&ctx, &repo_info);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].repo_name, "cargo");
    }
}