#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::process::Command;
    use std::path::Path;

    // Helper function to create a temporary git repository
    fn create_temp_git_repo() -> std::path::PathBuf {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        // Create a simple file
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        // Add and commit the file
        Command::new("git")
            .args(&["add", "test.txt"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");
        
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");
        
        repo_path.to_path_buf()
    }

    // Helper function to create a temporary git submodule
    fn create_temp_submodule_repo() -> std::path::PathBuf {
        let temp_dir = tempdir().unwrap();
        let parent_repo = temp_dir.path().join("parent");
        let submodule_repo = temp_dir.path().join("submodule");
        
        // Create parent repository
        Command::new("git")
            .args(&["init"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to initialize parent repo");
        
        // Create submodule repository
        Command::new("git")
            .args(&["init"])
            .current_dir(&submodule_repo)
            .output()
            .expect("Failed to initialize submodule repo");
        
        // Add file to submodule
        let submodule_file = submodule_repo.join("submodule.txt");
        fs::write(&submodule_file, "submodule content").unwrap();
        
        Command::new("git")
            .args(&["add", "submodule.txt"])
            .current_dir(&submodule_repo)
            .output()
            .expect("Failed to add submodule file");
        
        Command::new("git")
            .args(&["commit", "-m", "Submodule commit"])
            .current_dir(&submodule_repo)
            .output()
            .expect("Failed to submodule commit");
        
        // Add submodule to parent
        Command::new("git")
            .args(&["submodule", "add", "../submodule", "submodule"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to add submodule");
        
        Command::new("git")
            .args(&["commit", "-m", "Add submodule"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to commit submodule");
        
        parent_repo.to_path_buf()
    }

    // Helper function to create a repository with specific programming language files
    fn create_repo_with_languages() -> std::path::PathBuf {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        // Create files for different programming languages
        let rust_file = repo_path.join("src/main.rs");
        fs::write(&rust_file, "fn main() { println!(\"Hello, World!\"); }").unwrap();
        
        let python_file = repo_path.join("script.py");
        fs::write(&python_file, "print('Hello, World!')").unwrap();
        
        let go_file = repo_path.join("main.go");
        fs::write(&go_file, "package main\n\nfunc main() {\n    println(\"Hello, World!\")\n}").unwrap();
        
        let js_file = repo_path.join("app.js");
        fs::write(&js_file, "console.log('Hello, World!');").unwrap();
        
        let c_file = repo_path.join("program.c");
        fs::write(&c_file, "#include <stdio.h>\n\nint main() {\n    printf(\"Hello, World!\\n\");\n    return 0;\n}").unwrap();
        
        let toml_file = repo_path.join("Cargo.toml");
        fs::write(&toml_file, "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
        
        let json_file = repo_path.join("package.json");
        fs::write(&json_file, "{\"name\": \"test\", \"version\": \"1.0.0\"}").unwrap();
        
        let go_mod_file = repo_path.join("go.mod");
        fs::write(&go_mod_file, "module test\n\ngo 1.19\n").unwrap();
        
        // Create a CBOR file
        let cbor_file = repo_path.join("data.cbor");
        fs::write(&cbor_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        // Add and commit all files
        Command::new("git")
            .args(&["add", "."])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add files");
        
        Command::new("git")
            .args(&["commit", "-m", "Add language files"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");
        
        repo_path.to_path_buf()
    }

    // Helper function to create a repository with submodules
    fn create_repo_with_submodules() -> std::path::PathBuf {
        let temp_dir = tempdir().unwrap();
        let parent_repo = temp_dir.path().join("parent");
        let submodule1 = temp_dir.path().join("submodule1");
        let submodule2 = temp_dir.path().join("submodule2");
        
        // Create parent repository
        Command::new("git")
            .args(&["init"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to initialize parent repo");
        
        // Create submodule repositories
        for (name, path) in [("submodule1", &submodule1), ("submodule2", &submodule2)] {
            Command::new("git")
                .args(&["init"])
                .current_dir(path)
                .output()
                .expect("Failed to initialize submodule repo");
            
            let file = path.join("file.txt");
            fs::write(&file, format!("Content for {}", name)).unwrap();
            
            Command::new("git")
                .args(&["add", "file.txt"])
                .current_dir(path)
                .output()
                .expect("Failed to add submodule file");
            
            Command::new("git")
                .args(&["commit", "-m", format!("Initial commit for {}", name)])
                .current_dir(path)
                .output()
                .expect("Failed to submodule commit");
        }
        
        // Add submodules to parent
        Command::new("git")
            .args(&["submodule", "add", "../submodule1", "submodule1"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to add submodule1");
        
        Command::new("git")
            .args(&["submodule", "add", "../submodule2", "submodule2"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to add submodule2");
        
        Command::new("git")
            .args(&["commit", "-m", "Add submodules"])
            .current_dir(&parent_repo)
            .output()
            .expect("Failed to commit submodules");
        
        parent_repo.to_path_buf()
    }

    // ============================================================
    // Git Repository Discovery Tests
    // ============================================================

    #[test]
    fn test_discover_git_repos_single_repo() {
        let repo_path = create_temp_git_repo();
        let repos = discover_git_repos(&repo_path).unwrap();
        
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].path, repo_path);
        assert!(!repos[0].url.is_some()); // No remote URL in this test
        assert!(repos[0].branch.is_some());
        assert!(repos[0].commit.is_some());
        assert!(!repos[0].is_submodule);
        assert!(repos[0].submodules.is_empty());
    }

    #[test]
    fn test_discover_git_repos_multiple_repos() {
        let temp_dir = tempdir().unwrap();
        let repo1 = temp_dir.path().join("repo1");
        let repo2 = temp_dir.path().join("repo2");
        
        // Create two repositories
        for path in &[&repo1, &repo2] {
            Command::new("git")
                .args(&["init"])
                .current_dir(path)
                .output()
                .expect("Failed to initialize git repo");
            
            let test_file = path.join("test.txt");
            fs::write(&test_file, "test").unwrap();
            
            Command::new("git")
                .args(&["add", "test.txt"])
                .current_dir(path)
                .output()
                .expect("Failed to add file");
            
            Command::new("git")
                .args(&["commit", "-m", "test"])
                .current_dir(path)
                .output()
                .expect("Failed to commit");
        }
        
        let repos = discover_git_repos(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 2);
        
        // Check that both repositories are found
        let repo_paths: Vec<_> = repos.iter().map(|r| r.path.clone()).collect();
        assert!(repo_paths.contains(&repo1));
        assert!(repo_paths.contains(&repo2));
    }

    #[test]
    fn test_discover_git_repos_no_repos() {
        let temp_dir = tempdir().unwrap();
        let repos = discover_git_repos(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_discover_git_repos_nonexistent_path() {
        let result = discover_git_repos(Path::new("/non/existent/path"));
        assert!(result.is_err());
    }

    // ============================================================
    // Git Repository Metadata Tests
    // ============================================================

    #[test]
    fn test_scan_git_repo_basic() {
        let repo_path = create_temp_git_repo();
        let repo = scan_git_repo(&repo_path).unwrap();
        
        assert_eq!(repo.path, repo_path);
        assert!(!repo.is_submodule);
        assert!(repo.branch.is_some());
        assert!(repo.commit.is_some());
        assert!(repo.submodules.is_empty());
    }

    #[test]
    fn test_scan_git_repo_with_url() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        // Add remote URL
        let config_content = "[remote \"origin\"]\n\turl = https://github.com/test/repo.git\n";
        let config_file = repo_path.join(".git").join("config");
        fs::write(&config_file, config_content).unwrap();
        
        let repo = scan_git_repo(repo_path).unwrap();
        assert_eq!(repo.url, Some("https://github.com/test/repo.git".to_string()));
    }

    #[test]
    fn test_scan_git_repo_as_submodule() {
        let submodule_repo = create_temp_submodule_repo();
        let submodule_path = submodule_repo.join("submodule");
        
        // Create .git file that points to gitdir (simulating submodule)
        let git_file = submodule_path.join(".git");
        let gitdir_content = "gitdir: ../../.git/modules/submodule";
        fs::write(&git_file, gitdir_content).unwrap();
        
        let repo = scan_git_repo(&submodule_path).unwrap();
        assert!(repo.is_submodule);
    }

    #[test]
    fn test_scan_git_repo_with_submodules() {
        let parent_repo = create_repo_with_submodules();
        let repo = scan_git_repo(&parent_repo).unwrap();
        
        assert_eq!(repo.submodules.len(), 2);
        assert!(!repo.submodules.is_empty());
        
        // Check that submodules have correct paths
        let submodule_paths: Vec<_> = repo.submodules.iter().map(|s| s.path.clone()).collect();
        assert!(submodule_paths.contains(&parent_repo.join("submodule1")));
        assert!(submodule_paths.contains(&parent_repo.join("submodule2")));
    }

    // ============================================================
    // Language Detection Tests
    // ============================================================

    #[test]
    fn test_detect_languages_rust() {
        let repo_path = create_repo_with_languages();
        let languages = detect_languages(&repo_path).unwrap();
        
        assert!(languages.contains("Rust"));
        assert!(languages.contains("Python"));
        assert!(languages.contains("Go"));
        assert!(languages.contains("JavaScript"));
        assert!(languages.contains("C/C++"));
        assert!(languages.contains("TOML"));
        assert!(languages.contains("JSON"));
        assert!(languages.contains("CBOR"));
    }

    #[test]
    fn test_detect_languages_no_languages() {
        let temp_dir = tempdir().unwrap();
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert_eq!(languages.len(), 0);
    }

    #[test]
    fn test_detect_languages_build_files() {
        let temp_dir = tempdir().unwrap();
        
        // Create build files without source files
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(temp_dir.path().join("package.json"), "{\"name\": \"test\"}").unwrap();
        fs::write(temp_dir.path().join("go.mod"), "module test").unwrap();
        
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert!(languages.contains("Rust"));
        assert!(languages.contains("JavaScript/Node.js"));
        assert!(languages.contains("Go"));
    }

    #[test]
    fn test_detect_languages_cbor_files() {
        let temp_dir = tempdir().unwrap();
        
        // Create CBOR files
        fs::write(temp_dir.path().join("data.cbor"), vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        fs::write(temp_dir.path().join("other.cbor"), vec![0x1a, 0x00, 0x00, 0x01, 0x2c]).unwrap();
        
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert!(languages.contains("CBOR"));
    }

    #[test]
    fn test_detect_languages_case_sensitivity() {
        let temp_dir = tempdir().unwrap();
        
        // Test case sensitivity of file extensions
        fs::write(temp_dir.path().join("test.RS"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.Py"), "print('test')").unwrap();
        
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert!(languages.contains("Rust"));
        assert!(languages.contains("Python"));
    }

    // ============================================================
    // Submodule Detection Tests
    // ============================================================

    #[test]
    fn test_find_submodules() {
        let parent_repo = create_repo_with_submodules();
        let submodules = find_submodules(&parent_repo).unwrap();
        
        assert_eq!(submodules.len(), 2);
        
        // Check submodule paths
        let submodule_paths: Vec<_> = submodules.iter().map(|s| s.path.clone()).collect();
        assert!(submodule_paths.contains(&parent_repo.join("submodule1")));
        assert!(submodule_paths.contains(&parent_repo.join("submodule2")));
    }

    #[test]
    fn test_find_submodules_no_submodules() {
        let repo_path = create_temp_git_repo();
        let submodules = find_submodules(&repo_path).unwrap();
        assert_eq!(submodules.len(), 0);
    }

    #[test]
    fn test_find_submodules_missing_gitmodules() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository without .gitmodules
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        let submodules = find_submodules(repo_path).unwrap();
        assert_eq!(submodules.len(), 0);
    }

    // ============================================================
    // Git Command Tests
    // ============================================================

    #[test]
    fn test_get_git_branch() {
        let repo_path = create_temp_git_repo();
        let branch = get_git_branch(&repo_path).unwrap();
        
        assert!(!branch.is_empty());
        assert_ne!(branch, "unknown");
    }

    #[test]
    fn test_get_git_commit() {
        let repo_path = create_temp_git_repo();
        let commit = get_git_commit(&repo_path).unwrap();
        
        assert!(!commit.is_empty());
        assert_ne!(commit, "unknown");
        // Commit should be 40 characters (SHA-1 hash)
        assert_eq!(commit.len(), 40);
    }

    #[test]
    fn test_get_git_branch_invalid_repo() {
        let result = get_git_branch(Path::new("/non/existent/repo"));
        assert!(result.is_ok()); // Should return "unknown" without error
        assert_eq!(result.unwrap(), "unknown");
    }

    #[test]
    fn test_get_git_commit_invalid_repo() {
        let result = get_git_commit(Path::new("/non/existent/repo"));
        assert!(result.is_ok()); // Should return "unknown" without error
        assert_eq!(result.unwrap(), "unknown");
    }

    // ============================================================
    // Scanner Integration Tests
    // ============================================================

    #[test]
    fn test_run_scanner_success() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "test_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "echo 'test successful' > {output}/test_result.txt".to_string(),
            output_dir: "test_output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], test_file);
        
        // Check that command was executed
        let output_file = temp_dir.path().join("test_output").join("test_result.txt");
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        assert_eq!(content, "test successful\n");
    }

    #[test]
    fn test_run_scanner_no_matching_files() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "not cbor").unwrap();
        
        let config = ScannerConfig {
            name: "cbor_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "echo 'test'".to_string(),
            output_dir: "output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_run_scanner_command_failure() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "test_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "exit 1".to_string(), // Command that will fail
            output_dir: "output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path());
        // Command should still return the files it found, even if execution failed
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_run_scanner_output_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "test_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "echo 'test'".to_string(),
            output_dir: "nested/output/dir".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        
        // Check that output directory was created
        let output_dir = temp_dir.path().join("nested").join("output").join("dir");
        assert!(output_dir.exists());
    }

    // ============================================================
    // Complete Scanner Tests
    // ============================================================

    #[test]
    fn test_scan_all_single_repo() {
        let repo_path = create_repo_with_languages();
        let repos = vec![scan_git_repo(&repo_path).unwrap()];
        
        let config = WorkflowConfig {
            scanners: vec![
                ScannerConfig {
                    name: "test_scanner".to_string(),
                    extensions: vec!["txt".to_string()],
                    command: "echo 'test' > {output}/test.txt".to_string(),
                    output_dir: "test_output".to_string(),
                }
            ],
            output_dir: "scan_output".to_string(),
        };
        
        let result = scan_all(&repos, &config, repo_path.parent().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        
        let key = format!("{}:test_scanner", repo_path.display());
        assert!(result.contains_key(&key));
        assert_eq!(result[&key].len(), 0); // No .txt files in this repo
    }

    #[test]
    fn test_scan_all_multiple_repos() {
        let temp_dir = tempdir().unwrap();
        let repo1 = temp_dir.path().join("repo1");
        let repo2 = temp_dir.path().join("repo2");
        
        // Create two repositories
        for path in &[&repo1, &repo2] {
            Command::new("git")
                .args(&["init"])
                .current_dir(path)
                .output()
                .expect("Failed to initialize git repo");
            
            let test_file = path.join("test.txt");
            fs::write(&test_file, "test").unwrap();
            
            Command::new("git")
                .args(&["add", "test.txt"])
                .current_dir(path)
                .output()
                .expect("Failed to add file");
            
            Command::new("git")
                .args(&["commit", "-m", "test"])
                .current_dir(path)
                .output()
                .expect("Failed to commit");
        }
        
        let repos = vec![
            scan_git_repo(&repo1).unwrap(),
            scan_git_repo(&repo2).unwrap(),
        ];
        
        let config = WorkflowConfig {
            scanners: vec![
                ScannerConfig {
                    name: "txt_scanner".to_string(),
                    extensions: vec!["txt".to_string()],
                    command: "echo 'found txt' > {output}/found.txt".to_string(),
                    output_dir: "txt_output".to_string(),
                }
            ],
            output_dir: "scan_output".to_string(),
        };
        
        let result = scan_all(&repos, &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 2);
        
        // Check that both repositories were scanned
        for repo in &[repo1, repo2] {
            let key = format!("{}:txt_scanner", repo.display());
            assert!(result.contains_key(&key));
        }
    }

    // ============================================================
    // CLI Interface Tests
    // ============================================================

    #[test]
    fn test_cli_discover_command() {
        use clap::Parser;
        
        let repo_path = create_temp_git_repo();
        
        let args = Args::try_parse_from(&[
            "git_scanner",
            "discover",
            "-i", repo_path.to_str().unwrap(),
            "-o", "test_output.json",
            "--recursive"
        ]);
        
        assert!(args.is_ok());
        let args = args.unwrap();
        
        if let Command::Discover { root, output, recursive } = args.command {
            assert_eq!(root, repo_path);
            assert_eq!(output, PathBuf::from("test_output.json"));
            assert!(recursive);
        } else {
            panic!("Expected Discover command");
        }
    }

    #[test]
    fn test_cli_scan_command() {
        use clap::Parser;
        
        let repo_path = create_temp_git_repo();
        
        let args = Args::try_parse_from(&[
            "git_scanner",
            "scan",
            "-i", repo_path.to_str().unwrap(),
            "-c", "scanner_config.json",
            "-o", "scan_output"
        ]);
        
        assert!(args.is_ok());
        let args = args.unwrap();
        
        if let Command::Scan { input, config, output } = args.command {
            assert_eq!(input, repo_path);
            assert_eq!(config, PathBuf::from("scanner_config.json"));
            assert_eq!(output, PathBuf::from("scan_output"));
        } else {
            panic!("Expected Scan command");
        }
    }

    #[test]
    fn test_cli_list_languages_command() {
        use clap::Parser;
        
        let repo_path = create_repo_with_languages();
        
        let args = Args::try_parse_from(&[
            "git_scanner",
            "list-languages",
            "-i", repo_path.to_str().unwrap(),
            "-o", "languages.json"
        ]);
        
        assert!(args.is_ok());
        let args = args.unwrap();
        
        if let Command::ListLanguages { input, output } = args.command {
            assert_eq!(input, repo_path);
            assert_eq!(output, Some(PathBuf::from("languages.json")));
        } else {
            panic!("Expected ListLanguages command");
        }
    }

    #[test]
    fn test_cli_default_values() {
        use clap::Parser;
        
        let args = Args::try_parse_from(&[
            "git_scanner",
            "discover",
            "-i", "."
        ]);
        
        assert!(args.is_ok());
        let args = args.unwrap();
        
        if let Command::Discover { root, output, recursive } = args.command {
            assert_eq!(root, PathBuf::from("."));
            assert_eq!(output, PathBuf::from("repos.json"));
            assert!(!recursive); // default is false
        } else {
            panic!("Expected Discover command");
        }
    }

    // ============================================================
    // Error Handling Tests
    // ============================================================

    #[test]
    fn test_error_handling_invalid_git_repo() {
        let temp_dir = temp_dir().unwrap();
        let result = scan_git_repo(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_permission_denied() {
        // This test would require creating files with no read permissions
        // which is difficult to do safely in unit tests
        // For now, we'll skip this test
    }

    #[test]
    fn test_error_handling_malformed_gitmodules() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        // Create malformed .gitmodules file
        let gitmodules_content = "path = invalid\nurl = https://github.com/test/repo.git\n";
        let gitmodules_file = repo_path.join(".gitmodules");
        fs::write(&gitmodules_file, gitmodules_content).unwrap();
        
        let submodules = find_submodules(repo_path).unwrap();
        assert_eq!(submodules.len(), 0); // Should handle malformed file gracefully
    }

    // ============================================================
    // Performance Tests
    // ============================================================

    #[test]
    fn test_discover_git_repos_performance() {
        let temp_dir = tempdir().unwrap();
        
        // Create multiple repositories
        for i in 0..10 {
            let repo_path = temp_dir.path().join(format!("repo{}", i));
            fs::create_dir_all(&repo_path).unwrap();
            
            // Initialize git repository
            Command::new("git")
                .args(&["init"])
                .current_dir(&repo_path)
                .output()
                .expect("Failed to initialize git repo");
            
            let test_file = repo_path.join("test.txt");
            fs::write(&test_file, format!("test {}", i)).unwrap();
            
            Command::new("git")
                .args(&["add", "test.txt"])
                .current_dir(&repo_path)
                .output()
                .expect("Failed to add file");
            
            Command::new("git")
                .args(&["commit", "-m", format!("test {}", i)])
                .current_dir(&repo_path)
                .output()
                .expect("Failed to commit");
        }
        
        let start = std::time::Instant::now();
        let repos = discover_git_repos(temp_dir.path()).unwrap();
        let duration = start.elapsed();
        
        assert_eq!(repos.len(), 10);
        assert!(duration.as_millis() < 1000, "Should discover 10 repos within 1 second");
    }

    #[test]
    fn test_language_detection_performance() {
        let repo_path = create_repo_with_languages();
        
        let start = std::time::Instant::now();
        let languages = detect_languages(&repo_path).unwrap();
        let duration = start.elapsed();
        
        assert!(languages.len() > 5, "Should detect multiple languages");
        assert!(duration.as_millis() < 1000, "Should detect languages within 1 second");
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn test_empty_repository() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repository without any files
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");
        
        let repo = scan_git_repo(repo_path).unwrap();
        assert_eq!(repo.path, repo_path);
        assert!(repo.languages.is_empty());
        assert!(repo.submodules.is_empty());
    }

    #[test]
    fn test_repository_with_only_submodules() {
        let parent_repo = create_repo_with_submodules();
        let repo = scan_git_repo(&parent_repo).unwrap();
        
        assert_eq!(repo.submodules.len(), 2);
        assert!(repo.languages.is_empty()); // No source files in parent
    }

    #[test]
    fn test_large_number_of_languages() {
        let temp_dir = tempdir().unwrap();
        
        // Create files for many different programming languages
        let language_files = [
            ("test.rs", "Rust"),
            ("test.py", "Python"),
            ("test.go", "Go"),
            ("test.js", "JavaScript"),
            ("test.ts", "TypeScript"),
            ("test.java", "Java"),
            ("test.cpp", "C++"),
            ("test.c", "C"),
            ("test.h", "C/C++"),
            ("test.rb", "Ruby"),
            ("test.php", "PHP"),
            ("test.swift", "Swift"),
            ("test.kt", "Kotlin"),
            ("test.scala", "Scala"),
            ("test.hs", "Haskell"),
            ("test.ml", "OCaml"),
            ("test.erl", "Erlang"),
            ("test.exs", "Elixir"),
            ("test.clj", "Clojure"),
            ("test.lisp", "Lisp"),
            ("test.sh", "Shell"),
            ("test.zsh", "Zsh"),
            ("test.toml", "TOML"),
            ("test.yaml", "YAML"),
            ("test.yml", "YAML"),
            ("test.json", "JSON"),
            ("test.cbor", "CBOR"),
        ];
        
        for (filename, _) in &language_files {
            let file = temp_dir.path().join(filename);
            fs::write(&file, "// test file").unwrap();
        }
        
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert!(languages.len() >= 20, "Should detect many languages");
    }

    #[test]
    fn test_git_scanner_with_real_git_commands() {
        let repo_path = create_temp_git_repo();
        
        // Test that we can actually run git commands
        let branch = get_git_branch(&repo_path).unwrap();
        let commit = get_git_commit(&repo_path).unwrap();
        
        assert!(!branch.is_empty());
        assert!(!commit.is_empty());
        assert_eq!(commit.len(), 40); // SHA-1 hash
    }

    // ============================================================
    // Integration with External Commands
    // ============================================================

    #[test]
    fn test_scanner_with_external_command() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "external_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "wc -c {dir}/* > {output}/count.txt".to_string(),
            output_dir: "external_output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        
        // Check that external command was executed
        let output_file = temp_dir.path().join("external_output").join("count.txt");
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("test.cbor"));
    }

    #[test]
    fn test_scanner_command_template_substitution() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "template_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "echo 'dir: {dir}' > {output}/dir.txt && echo 'output: {output}' > {output}/output.txt".to_string(),
            output_dir: "template_output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        
        // Check template substitution
        let dir_file = temp_dir.path().join("template_output").join("dir.txt");
        let output_file = temp_dir.path().join("template_output").join("output.txt");
        
        let dir_content = fs::read_to_string(&dir_file).unwrap();
        let output_content = fs::read_to_string(&output_file).unwrap();
        
        assert!(dir_content.contains(&temp_dir.path().to_string_lossy()));
        assert!(output_content.contains(&temp_dir.path().join("template_output").to_string_lossy()));
    }
}