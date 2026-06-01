use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_temp_directory_creation() {
        let temp_dir = tempdir().unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_cargo.toml");

        let content = r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
        fs::write(&file_path, content).unwrap();
        assert!(file_path.exists());

        let read_back = fs::read_to_string(&file_path).unwrap();
        assert!(read_back.contains("test-crate"));
        assert!(read_back.contains("serde"));
    }

    #[test]
    fn test_path_manipulation() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        let nested = base_path.join("crates").join("my-crate").join("src");
        fs::create_dir_all(&nested).unwrap();
        assert!(nested.exists());

        let lib_rs = nested.join("lib.rs");
        fs::write(&lib_rs, "// test").unwrap();
        assert!(lib_rs.exists());
    }

    #[test]
    fn test_error_handling() {
        let nonexistent = std::path::PathBuf::from("/nonexistent/path/cargo.toml");
        let result = fs::read_to_string(&nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_dir_entry_filtering() {
        let temp_dir = tempdir().unwrap();

        fs::create_dir(temp_dir.path().join("crate-a")).unwrap();
        fs::create_dir(temp_dir.path().join("crate-b")).unwrap();
        fs::write(temp_dir.path().join("README.md"), "# test").unwrap();

        let entries: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .flatten()
            .collect();

        let dirs: Vec<_> = entries.iter()
            .filter(|e| e.path().is_dir())
            .collect();
        assert_eq!(dirs.len(), 2);
    }
}
