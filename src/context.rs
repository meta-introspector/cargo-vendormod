use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use toml_edit::DocumentMut;

#[derive(Clone)]
pub struct AppContext {
    pub git_executable_path: PathBuf,
    pub root_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub submodules_dir: PathBuf,
    pub mirrors_path: PathBuf,
    pub vendor_dir: PathBuf,
    pub target_branch: String,
    pub create_version_branches: bool,
    pub version_branch_format: String,
    pub dry_run: bool,
    pub verbose: bool,
    pub temp_symlink: Option<PathBuf>,
}

// Default configuration values (fallback when not specified in Cargo.toml or CLI)
const DEFAULT_SUBMODULES_DIR: &str = "submodules";
const DEFAULT_MIRRORS_PATH: &str = "/home/mdupont/git/host";
const DEFAULT_VENDOR_DIR: &str = "vendor";
const DEFAULT_TARGET_BRANCH: &str = "feature/CRQ-016-nixify";
const DEFAULT_VERSION_BRANCH_FORMAT: &str = "v{}";
const DEFAULT_CREATE_VERSION_BRANCHES: bool = false;
const DEFAULT_MANIFEST_PATH: &str = "/mnt/data1/nix/vendor/rust/cargo2nix/CargoRepSyncCargo.toml";

#[derive(Default)]
struct VendormodConfig {
    vendor_dir: Option<PathBuf>,
    submodules_dir: Option<PathBuf>,
    mirrors_dir: Option<PathBuf>,
    default_manifest_path: Option<PathBuf>,
    default_target_branch: Option<String>,
    version_branch_format: Option<String>,
    create_version_branches: Option<bool>,
}

impl AppContext {
    pub fn from_args(args: &crate::Args) -> Result<Self> {
        // Locate vendormod's own Cargo.toml for metadata (git_path, etc).
        // In development, CARGO_MANIFEST_DIR points to source. In Nix installations,
        // the source may not be available, so fall back to safe defaults.
        let compile_time_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let vendormod_cargo_toml_path = compile_time_manifest_dir.join("Cargo.toml");

        // Try to read and parse vendormod's own Cargo.toml, if present.
        let vendormod_cargo_toml_doc: Option<DocumentMut> = if vendormod_cargo_toml_path.exists() {
            match fs::read_to_string(&vendormod_cargo_toml_path) {
                Ok(content) => content.parse::<DocumentMut>().ok(),
                Err(_) => None,
            }
        } else {
            None
        };

        // Extract git executable path from metadata, falling back to "git" in PATH.
        let git_executable_path = if let Some(ref doc) = vendormod_cargo_toml_doc {
            doc.get("package")
                .and_then(|item| item.as_table())
                .and_then(|table| table.get("metadata"))
                .and_then(|item| item.as_table())
                .and_then(|table| table.get("repo-manager"))
                .and_then(|item| item.as_table())
                .and_then(|table| table.get("git_path"))
                .and_then(|item| item.as_str())
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("git"))
        } else {
            PathBuf::from("git")
        };

        // Extract vendormod configuration table (may be None).
        let vendormod_config = vendormod_cargo_toml_doc
            .as_ref()
            .and_then(|doc| {
                doc.get("package")
                    .and_then(|item| item.as_table())
                    .and_then(|table| table.get("metadata"))
                    .and_then(|item| item.as_table())
                    .and_then(|table| table.get("cargo-vendormod"))
                    .and_then(|item| item.as_table())
            });

        let root_dir = args.root_dir
            .canonicalize()
            .context("Failed to canonicalize root_dir")?;

        // Determine manifest path
        let mut manifest_path = if let Some(ref p) = args.manifest_path {
            p.clone()
        } else {
            let standard = root_dir.join("Cargo.toml");
            if standard.exists() {
                standard
            } else {
                let pattern = format!("{}/{}Cargo.toml", root_dir.display(), if root_dir.as_os_str().to_string_lossy().ends_with('/') { "" } else { "*" });
                match glob::glob(&pattern) {
                    Ok(entries) => entries
                        .filter_map(|e| e.ok())
                        .find(|p| {
                            p.file_name().and_then(|n| n.to_str()).map(|s| !s.starts_with('.') && !s.starts_with('#')).unwrap_or(false)
                        })
                        .unwrap_or(standard),
                    Err(_) => standard,
                }
            }
        };

        // Ensure cargo will accept the manifest: it must be named Cargo.toml
        let temp_symlink = if manifest_path.file_name() != Some(std::ffi::OsStr::new("Cargo.toml")) {
            let link = root_dir.join("Cargo.toml");
            if !link.exists() {
                std::os::unix::fs::symlink(&manifest_path, &link)
                    .context("Failed to create symlink for manifest")?;
                Some(link.clone())
            } else {
                None
            }
        } else {
            None
        };

        // If we created a symlink, use it as the manifest path for cargo
        if temp_symlink.is_some() {
            manifest_path = root_dir.join("Cargo.toml");
        }

        // Helper to extract string value from metadata table
        let get_str = |table: Option<&toml_edit::Table>, key: &str| -> Option<String> {
            table.and_then(|t| t.get(key)).and_then(|v| v.as_str()).map(|s| s.to_string())
        };

        // Helper to extract PathBuf from metadata table
        let _get_path = |table: Option<&toml_edit::Table>, key: &str| -> Option<PathBuf> {
            get_str(table, key).map(PathBuf::from)
        };

        // Helper to extract bool from metadata table
        let get_bool = |table: Option<&toml_edit::Table>, key: &str| -> Option<bool> {
            table.and_then(|t| t.get(key)).and_then(|v| v.as_bool())
        };

        // Merge configuration with fallback order: CLI > package metadata > hardcoded defaults
        let submodules_dir = if let Some(ref p) = args.submodules_path {
            // CLI-provided path: use absolute if absolute, otherwise relative to root_dir
            if p.is_absolute() {
                p.clone()
            } else {
                root_dir.join(p)
            }
        } else if let Some(path_str) = get_str(vendormod_config, "submodules-dir") {
            let p = PathBuf::from(path_str);
            if p.is_absolute() { p } else { root_dir.join(p) }
        } else {
            root_dir.join(DEFAULT_SUBMODULES_DIR)
        };

        let mirrors_path = if let Some(ref p) = args.mirrors_path {
            if p.is_absolute() {
                p.clone()
            } else {
                root_dir.join(p)
            }
        } else if let Some(path_str) = get_str(vendormod_config, "bare-mirrors-dir") {
            let p = PathBuf::from(path_str);
            if p.is_absolute() { p } else { root_dir.join(p) }
        } else {
            PathBuf::from(DEFAULT_MIRRORS_PATH)
        };

        let vendor_dir = if let Some(ref p) = args.vendor_dir {
            if p.is_absolute() {
                p.clone()
            } else {
                root_dir.join(p)
            }
        } else if let Some(path_str) = get_str(vendormod_config, "vendor-dir") {
            let p = PathBuf::from(path_str);
            if p.is_absolute() { p } else { root_dir.join(p) }
        } else {
            root_dir.join(DEFAULT_VENDOR_DIR)
        };

        let target_branch = args.target_branch.clone().or_else(|| {
            get_str(vendormod_config, "default-target-branch")
                .or_else(|| Some(DEFAULT_TARGET_BRANCH.to_string()))
        }).unwrap_or_else(|| DEFAULT_TARGET_BRANCH.to_string());

        let version_branch_format = args.version_branch_format.clone().or_else(|| {
            get_str(vendormod_config, "version-branch-format")
                .or_else(|| Some(DEFAULT_VERSION_BRANCH_FORMAT.to_string()))
        }).unwrap_or_else(|| DEFAULT_VERSION_BRANCH_FORMAT.to_string());

        let create_version_branches = args.create_version_branches.unwrap_or_else(|| {
            get_bool(vendormod_config, "create-version-branches")
                .unwrap_or(DEFAULT_CREATE_VERSION_BRANCHES)
        });

        Ok(AppContext {
            git_executable_path,
            root_dir,
            manifest_path,
            submodules_dir,
            mirrors_path,
            vendor_dir,
            target_branch,
            create_version_branches,
            version_branch_format,
            dry_run: args.dry_run,
            verbose: args.verbose,
            temp_symlink,
        })
    }
}

impl Drop for AppContext {
    fn drop(&mut self) {
        if let Some(ref p) = self.temp_symlink {
            let _ = fs::remove_file(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_app_context_default_paths() {
        let dir = tempdir().unwrap();
        let args = crate::Args::parse_from(&["test", "--root-dir", dir.path().to_str().unwrap()]);
        let ctx = AppContext::from_args(&args).unwrap();
        assert!(ctx.root_dir.ends_with(dir.path().file_name().unwrap()));
    }

    #[test]
    fn test_app_context_absolute_paths() {
        let dir = tempdir().unwrap();
        let args = crate::Args::parse_from(&["test", 
            "--root-dir", dir.path().to_str().unwrap(),
            "--submodules-path", "/absolute/path",
            "--mirrors-path", "/absolute/mirrors",
        ]);
        let ctx = AppContext::from_args(&args).unwrap();
        assert_eq!(ctx.submodules_dir, PathBuf::from("/absolute/path"));
        assert_eq!(ctx.mirrors_path, PathBuf::from("/absolute/mirrors"));
    }

    #[test]
    fn test_app_context_relative_paths() {
        let dir = tempdir().unwrap();
        let args = crate::Args::parse_from(&["test",
            "--root-dir", dir.path().to_str().unwrap(),
            "--submodules-path", "relative/submodules",
        ]);
        let ctx = AppContext::from_args(&args).unwrap();
        assert!(ctx.submodules_dir.ends_with("relative/submodules"));
    }

    #[test]
    fn test_app_context_dry_run() {
        let dir = tempdir().unwrap();
        let args = crate::Args::parse_from(&["test", "--root-dir", dir.path().to_str().unwrap(), "--dry-run"]);
        let ctx = AppContext::from_args(&args).unwrap();
        assert!(ctx.dry_run);
    }
}