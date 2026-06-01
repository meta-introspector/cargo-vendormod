use cargo_vendormod::config::Config;
use cargo_vendormod::args::Args;
use std::path::PathBuf;

#[test]
fn test_config_default_target_branch() {
    let config = Config::default();
    assert_eq!(config.target_branch, "main");
}

#[test]
fn test_config_version_branch() {
    let config = Config::default();
    assert_eq!(config.version_branch("1.0.0"), "v1.0.0");
    let mut config = Config::default();
    config.version_branch_format = "release-{}".to_string();
    assert_eq!(config.version_branch("1.0.0"), "release-1.0.0");
}

#[test]
fn test_config_get_mirror_path() {
    let config = Config::default();
    let mirror = config.get_mirror_path("owner", "repo");
    assert!(mirror.ends_with("owner/repo.git"));
}

#[test]
fn test_config_get_submodule_path() {
    let config = Config::default();
    let sub = config.get_submodule_path("my-crate");
    assert_eq!(sub, PathBuf::from("submodules/my-crate"));
}

#[test]
fn test_args_has_verbose_flag() {
    let args = Args {
        verbose: false,
        root_dir: PathBuf::from("."),
        manifest_path: None,
        submodules_path: None,
        mirrors_path: None,
        vendor_dir: None,
        target_branch: None,
        create_version_branches: None,
        version_branch_format: None,
        dry_run: false,
        output_file: None,
        source_repo: PathBuf::from("."),
        include_dev: true,
        include_build: true,
        include_optional: true,
        fetch_crates_io_repos: false,
        command: None,
    };
    assert!(!args.verbose);
    assert!(args.include_dev);
}

#[test]
fn test_config_merge_sample() {
    let sample = cargo_vendormod::generate_sample_config();
    assert!(sample.contains("git-path"));
    assert!(sample.contains("vendor-dir"));
}

#[cfg(test)]
mod system_tests {
    use tempfile::tempdir;
    use std::path::PathBuf;

    #[test]
    fn test_temp_dir_is_empty() {
        let dir = tempdir().unwrap();
        let expected = dir.path().read_dir().unwrap().count();
        assert_eq!(expected, 0);
    }

    #[test]
    fn test_path_join_operations() {
        let base = PathBuf::from("/tmp/test");
        let joined = base.join("foo").join("bar");
        assert_eq!(joined, PathBuf::from("/tmp/test/foo/bar"));
    }

    #[test]
    fn test_string_trim_operations() {
        let s = "  hello world  ";
        assert_eq!(s.trim(), "hello world");
    }
}
