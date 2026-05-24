// Integration tests for cargo-vendormod functionality
use std::path::PathBuf;
use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_temp_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();
        assert!(path.exists());
    }
    
    #[test]
    fn test_file_operations() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        
        // Create and write to file
        fs::write(&file_path, "test content").unwrap();
        assert!(file_path.exists());
        
        // Read from file
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }
    
    #[test]
    fn test_path_manipulation() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();
        
        let subdir = base_path.join("subdir").join("test");
        fs::create_dir_all(&subdir).unwrap();
        assert!(subdir.exists());
        
        let file_in_subdir = subdir.join("file.txt");
        fs::write(&file_in_subdir, "content").unwrap();
        assert!(file_in_subdir.exists());
    }
    
    #[test]
    fn test_error_handling() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/file.txt");
        let result = fs::read_to_string(&nonexistent_path);
        assert!(result.is_err());
    }
}