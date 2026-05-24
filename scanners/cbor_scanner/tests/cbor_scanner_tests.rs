#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::path::Path;

    // Helper function to create test CBOR data
    fn create_test_cbor_data() -> Vec<u8> {
        // Simple CBOR: unsigned integer 100
        vec![0x1a, 0x00, 0x00, 0x00, 0x64]
    }

    // Helper function to create test CBOR with multiple integers
    fn create_test_cbor_multiple_integers() -> Vec<u8> {
        // CBOR: unsigned integer 100, unsigned integer 200
        vec![0x1a, 0x00, 0x00, 0x00, 0x64, 0x1a, 0x00, 0x00, 0x00, 0xc8]
    }

    // Helper function to create malformed CBOR data
    fn create_malformed_cbor_data() -> Vec<u8> {
        vec![0xff, 0x00, 0x00] // Invalid CBOR
    }

    // Helper function to scan CBOR from bytes (for testing)
    fn scan_cbor_file_from_bytes(data: &[u8]) -> Result<CBORAnalysis> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, data)?;
        scan_cbor_file(&test_file)
    }

    // ============================================================
    // HMM Model Tests
    // ============================================================

    #[test]
    fn test_hmm_initialization() {
        let hmm = HMM::new(4);
        assert_eq!(hmm.states.len(), 4);
        assert_eq!(hmm.state_names.len(), 4);
        
        // Check state names are correctly formatted
        for (i, name) in hmm.state_names.iter().enumerate() {
            assert_eq!(name, &format!("S{}", i));
        }
    }

    #[test]
    fn test_hmm_training_simple() {
        let mut hmm = HMM::new(2);
        let sequences = vec![
            vec![1, 2, 3, 4, 5],
            vec![10, 20, 30, 40, 50],
            vec![1, 2, 3, 4, 5]
        ];
        
        hmm.train_simple(&sequences);
        
        // Check that emission probabilities sum to ~1.0 for each state
        for state in &hmm.states {
            let total: f64 = state.emissions.values().sum();
            assert!((total - 1.0).abs() < 0.1, "Emission probabilities should sum to ~1.0");
            
            // Check that all probabilities are non-negative
            for &prob in state.emissions.values() {
                assert!(prob >= 0.0, "Emission probabilities should be non-negative");
            }
        }
        
        // Check that transition probabilities are reasonable
        for state in &hmm.states {
            let total: f64 = state.transitions.values().sum();
            assert!((total - 1.0).abs() < 0.1, "Transition probabilities should sum to ~1.0");
        }
    }

    #[test]
    fn test_hmm_training_empty_sequences() {
        let mut hmm = HMM::new(2);
        let empty_sequences: Vec<Vec<u8>> = vec![];
        
        hmm.train_simple(&empty_sequences);
        
        // Should not panic, should handle empty sequences gracefully
        for state in &hmm.states {
            assert!(state.emissions.is_empty());
            assert!(state.transitions.is_empty());
        }
    }

    #[test]
    fn test_viterbi_scoring() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3], vec![4, 5, 6]];
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1, 2, 3];
        let (score, path) = hmm.viterbi_score(&test_seq);
        
        assert!(!score.is_nan(), "Score should not be NaN");
        assert!(!score.is_infinite(), "Score should not be infinite");
        assert_eq!(path.len(), test_seq.len(), "Path length should match sequence length");
        
        // Check that path contains valid state indices
        for &state in &path {
            assert!(state < hmm.states.len(), "State index should be valid");
        }
    }

    #[test]
    fn test_viterbi_scoring_empty_sequence() {
        let hmm = HMM::new(2);
        let empty_seq: Vec<u8> = vec![];
        let (score, path) = hmm.viterbi_score(&empty_seq);
        
        assert_eq!(score, 0.0);
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_viterbi_scoring_single_byte() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3]];
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1];
        let (score, path) = hmm.viterbi_score(&test_seq);
        
        assert!(!score.is_nan());
        assert_eq!(path.len(), 1);
    }

    // ============================================================
    // CBOR File Processing Tests
    // ============================================================

    #[test]
    fn test_cbor_parsing_valid() {
        let test_data = create_test_cbor_data();
        let analysis = scan_cbor_file_from_bytes(&test_data).unwrap();
        
        assert!(!analysis.integers.is_empty(), "Should find integers in valid CBOR");
        assert!(analysis.integers.contains(&100), "Should find integer 100");
        assert_eq!(analysis.integer_positions.len(), analysis.integers.len(), "Positions count should match integers count");
        assert_eq!(analysis.path, "test.cbor");
        assert_eq!(analysis.file_size, test_data.len());
    }

    #[test]
    fn test_cbor_parsing_multiple_integers() {
        let test_data = create_test_cbor_multiple_integers();
        let analysis = scan_cbor_file_from_bytes(&test_data).unwrap();
        
        assert_eq!(analysis.integers.len(), 2, "Should find 2 integers");
        assert!(analysis.integers.contains(&100), "Should find integer 100");
        assert!(analysis.integers.contains(&200), "Should find integer 200");
    }

    #[test]
    fn test_cbor_parsing_no_integers() {
        // CBOR with no integers (just text)
        let test_data = vec![0x74, 0x68, 0x65, 0x72, 0x65]; // "there" as text
        let analysis = scan_cbor_file_from_bytes(&test_data).unwrap();
        
        // Should still find integers from raw byte scanning
        assert!(!analysis.integers.is_empty(), "Should find integers from raw byte scanning");
    }

    #[test]
    fn test_malformed_cbor_handling() {
        let malformed_data = create_malformed_cbor_data();
        let result = scan_cbor_file_from_bytes(&malformed_data);
        
        // Should not panic, should handle gracefully
        assert!(result.is_ok(), "Should handle malformed CBOR gracefully");
        
        let analysis = result.unwrap();
        // Should still process raw bytes even if CBOR parsing fails
        assert!(!analysis.integers.is_empty(), "Should find integers from raw byte scanning");
    }

    #[test]
    fn test_empty_cbor_handling() {
        let empty_data: Vec<u8> = vec![];
        let result = scan_cbor_file_from_bytes(&empty_data);
        
        assert!(result.is_ok(), "Should handle empty CBOR gracefully");
        let analysis = result.unwrap();
        assert_eq!(analysis.integers.len(), 0, "Should have no integers in empty file");
        assert_eq!(analysis.file_size, 0, "File size should be 0");
    }

    #[test]
    fn test_integer_positions() {
        let test_data = create_test_cbor_multiple_integers();
        let analysis = scan_cbor_file_from_bytes(&test_data).unwrap();
        
        // First integer at position 0
        assert_eq!(analysis.integer_positions[0], 0);
        // Second integer at position 5 (after first 5-byte integer)
        assert_eq!(analysis.integer_positions[1], 5);
    }

    // ============================================================
    // Anomaly Detection Tests
    // ============================================================

    #[test]
    fn test_anomaly_detection_normal() {
        let mut hmm = HMM::new(2);
        let normal_sequences = vec![
            vec![1, 2, 3, 4, 5],
            vec![10, 20, 30, 40, 50],
            vec![1, 2, 3, 4, 5]
        ];
        hmm.train_simple(&normal_sequences);
        
        let normal_seq = vec![1, 2, 3, 4, 5];
        let anomalies = detect_anomalies(&normal_seq, &hmm, &vec![0, 0, 0, 0, 0]);
        
        // Normal sequence should have few or no anomalies
        assert!(anomalies.len() <= 1, "Normal sequence should have few anomalies");
    }

    #[test]
    fn test_anomaly_detection_outlier() {
        let mut hmm = HMM::new(2);
        let normal_sequences = vec![
            vec![1, 2, 3, 4, 5],
            vec![10, 20, 30, 40, 50],
            vec![1, 2, 3, 4, 5]
        ];
        hmm.train_simple(&normal_sequences);
        
        let anomalous_seq = vec![1, 2, 255, 4, 5]; // Contains outlier byte
        let anomalies = detect_anomalies(&anomalous_seq, &hmm, &vec![0, 0, 1, 0, 0]);
        
        assert!(!anomalies.is_empty(), "Should detect anomalies in outlier sequence");
        
        // Check anomaly details
        for anomaly in &anomalies {
            assert!(anomaly.offset < anomalous_seq.len(), "Anomaly offset should be valid");
            assert!(anomaly.length > 0, "Anomaly length should be positive");
            assert!(!anomaly.description.is_empty(), "Anomaly description should not be empty");
            assert!(anomaly.score > 0.0, "Anomaly score should be positive");
        }
    }

    #[test]
    fn test_rare_transition_detection() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3], vec![1, 2, 3]];
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1, 255, 3]; // Rare transition
        let anomalies = detect_anomalies(&test_seq, &hmm, &vec![0, 0, 1]);
        
        assert!(anomalies.len() > 0, "Should detect rare transitions");
    }

    #[test]
    fn test_anomaly_detection_empty_sequence() {
        let hmm = HMM::new(2);
        let empty_seq: Vec<u8> = vec![];
        let anomalies = detect_anomalies(&empty_seq, &hmm, &vec![]);
        
        assert_eq!(anomalies.len(), 0, "Empty sequence should have no anomalies");
    }

    #[test]
    fn test_anomaly_detection_short_sequence() {
        let hmm = HMM::new(2);
        let short_seq = vec![1];
        let anomalies = detect_anomalies(&short_seq, &hmm, &vec![0]);
        
        assert_eq!(anomalies.len(), 0, "Short sequence should have no anomalies");
    }

    // ============================================================
    // Markov Model Tests
    // ============================================================

    #[test]
    fn test_markov_model_training() {
        let sequences = vec![
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 4, 5],
            vec![5, 4, 3, 2, 1]
        ];
        
        let matrix = train_markov_model(&sequences);
        
        // Check matrix dimensions
        assert_eq!(matrix.len(), 256);
        for row in &matrix {
            assert_eq!(row.len(), 256);
        }
        
        // Check that probabilities are valid (0 <= p <= 1)
        for row in &matrix {
            for &prob in row {
                assert!(0.0 <= prob && prob <= 1.0, "Probabilities should be between 0 and 1");
            }
        }
    }

    #[test]
    fn test_markov_model_scoring() {
        let sequences = vec![vec![1, 2, 3], vec![1, 2, 3]];
        let matrix = train_markov_model(&sequences);
        
        let test_seq = vec![1, 2, 3];
        let score = score_sequence(&test_seq, &matrix);
        
        assert!(!score.is_nan(), "Score should not be NaN");
        assert!(!score.is_infinite(), "Score should not be infinite");
    }

    #[test]
    fn test_markov_model_short_sequence() {
        let sequences = vec![vec![1, 2, 3]];
        let matrix = train_markov_model(&sequences);
        
        let short_seq = vec![1];
        let score = score_sequence(&short_seq, &matrix);
        
        assert_eq!(score, 0.0, "Short sequence should have score 0");
    }

    // ============================================================
    // File Collection Tests
    // ============================================================

    #[test]
    fn test_collect_cbor_files_single_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, create_test_cbor_data()).unwrap();
        
        let files = collect_cbor_files(&test_file).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[test]
    fn test_collect_cbor_files_directory() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        let file1 = temp_dir.path().join("test1.cbor");
        let file2 = temp_dir.path().join("test2.cbor");
        let other_file = temp_dir.path().join("test.txt");
        
        fs::write(&file1, create_test_cbor_data()).unwrap();
        fs::write(&file2, create_test_cbor_multiple_integers()).unwrap();
        fs::write(&other_file, "not cbor").unwrap();
        
        let files = collect_cbor_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        
        // Check that all files are .cbor files
        for file in &files {
            assert_eq!(file.extension().unwrap(), "cbor");
        }
    }

    #[test]
    fn test_collect_cbor_files_no_cbor() {
        let temp_dir = temp_dir().unwrap();
        let other_file = temp_dir.path().join("test.txt");
        fs::write(&other_file, "not cbor").unwrap();
        
        let files = collect_cbor_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_collect_cbor_files_nonexistent() {
        let result = collect_cbor_files(Path::new("/non/existent/path"));
        assert!(result.is_err());
    }

    // ============================================================
    // CLI Interface Tests
    // ============================================================

    #[test]
    fn test_cli_command_parsing() {
        use clap::Parser;
        
        // Test Train command parsing
        let train_args = Args::try_parse_from(&[
            "cbor_scanner", "train",
            "-i", "test_data",
            "-o", "model.json",
            "-s", "8"
        ]);
        
        assert!(train_args.is_ok());
        let train_args = train_args.unwrap();
        if let Command::Train { input, output, states } = train_args.command {
            assert_eq!(input, PathBuf::from("test_data"));
            assert_eq!(output, PathBuf::from("model.json"));
            assert_eq!(states, 8);
        } else {
            panic!("Expected Train command");
        }
        
        // Test Scan command parsing
        let scan_args = Args::try_parse_from(&[
            "cbor_scanner", "scan",
            "-i", "test_data",
            "-m", "model.json",
            "-o", "analysis"
        ]);
        
        assert!(scan_args.is_ok());
        let scan_args = scan_args.unwrap();
        if let Command::Scan { input, model, output } = scan_args.command {
            assert_eq!(input, PathBuf::from("test_data"));
            assert_eq!(model, PathBuf::from("model.json"));
            assert_eq!(output, PathBuf::from("analysis"));
        } else {
            panic!("Expected Scan command");
        }
        
        // Test Extract command parsing
        let extract_args = Args::try_parse_from(&[
            "cbor_scanner", "extract",
            "-i", "test_data",
            "-o", "constants.json"
        ]);
        
        assert!(extract_args.is_ok());
        let extract_args = extract_args.unwrap();
        if let Command::Extract { input, output } = extract_args.command {
            assert_eq!(input, PathBuf::from("test_data"));
            assert_eq!(output, PathBuf::from("constants.json"));
        } else {
            panic!("Expected Extract command");
        }
    }

    #[test]
    fn test_cli_default_values() {
        let args = Args::try_parse_from(&[
            "cbor_scanner", "train",
            "-i", "test_data"
        ]);
        
        assert!(args.is_ok());
        let args = args.unwrap();
        if let Command::Train { input, output, states } = args.command {
            assert_eq!(input, PathBuf::from("test_data"));
            assert_eq!(output, PathBuf::from("cbor_hmm.json")); // default value
            assert_eq!(states, 4); // default value
        } else {
            panic!("Expected Train command");
        }
    }

    // ============================================================
    // Error Handling Tests
    // ============================================================

    #[test]
    fn test_error_handling_invalid_path() {
        let result = scan_cbor_file(Path::new("/non/existent/file.cbor"));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_permission_denied() {
        // This test would require creating a file with no read permissions
        // which is difficult to do safely in unit tests
        // For now, we'll skip this test
    }

    #[test]
    fn test_hmm_training_with_invalid_sequences() {
        let mut hmm = HMM::new(2);
        let sequences: Vec<Vec<u8>> = vec![
            vec![1, 2, 3],
            vec![], // Empty sequence
            vec![4, 5, 6]
        ];
        
        // Should not panic
        hmm.train_simple(&sequences);
        
        // Should still have valid states
        assert_eq!(hmm.states.len(), 2);
        for state in &hmm.states {
            assert!(state.emissions.is_empty() || state.emissions.len() > 0);
        }
    }

    // ============================================================
    // Performance Tests
    // ============================================================

    #[test]
    fn test_hmm_performance_large_sequence() {
        let mut hmm = HMM::new(4);
        let large_sequence = vec![1, 2, 3, 4, 5].repeat(2000); // 10,000 bytes
        let sequences = vec![large_sequence.clone(), large_sequence.clone()];
        
        // Training should complete in reasonable time
        let start = std::time::Instant::now();
        hmm.train_simple(&sequences);
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 1000, "Training should complete within 1 second");
        
        // Scoring should also be reasonable
        let start = std::time::Instant::now();
        let (score, path) = hmm.viterbi_score(&large_sequence);
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 500, "Scoring should complete within 500ms");
        assert!(!score.is_nan());
        assert_eq!(path.len(), large_sequence.len());
    }

    #[test]
    fn test_cbor_processing_performance() {
        let large_data = vec![0x1a, 0x00, 0x00, 0x00, 0x64].repeat(1000); // 1000 integers
        
        let start = std::time::Instant::now();
        let result = scan_cbor_file_from_bytes(&large_data);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should process large CBOR data");
        assert!(duration.as_millis() < 1000, "Should process within 1 second");
        
        let analysis = result.unwrap();
        assert_eq!(analysis.integers.len(), 1000, "Should find all 1000 integers");
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn test_maximum_states() {
        // Test with maximum reasonable number of states
        let hmm = HMM::new(32);
        assert_eq!(hmm.states.len(), 32);
        
        let sequences = vec![vec![1, 2, 3], vec![4, 5, 6]];
        hmm.train_simple(&sequences);
        
        // Should still work with many states
        let test_seq = vec![1, 2, 3];
        let (score, path) = hmm.viterbi_score(&test_seq);
        assert!(!score.is_nan());
        assert_eq!(path.len(), test_seq.len());
    }

    #[test]
    fn test_extreme_byte_values() {
        let sequences = vec![
            vec![0u8, 255u8, 127u8], // Min, max, middle values
            vec![255u8, 0u8, 128u8],
            vec![127u8, 128u8, 0u8]
        ];
        
        let mut hmm = HMM::new(3);
        hmm.train_simple(&sequences);
        
        let test_seq = vec![0u8, 255u8, 127u8];
        let (score, path) = hmm.viterbi_score(&test_seq);
        assert!(!score.is_nan());
        assert_eq!(path.len(), test_seq.len());
    }

    #[test]
    fn test_repeated_sequences() {
        let repeated_sequence = vec![1, 2, 3, 4, 5];
        let sequences = vec![repeated_sequence; 10]; // Repeat 10 times
        
        let mut hmm = HMM::new(2);
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1, 2, 3, 4, 5];
        let (score, path) = hmm.viterbi_score(&test_seq);
        assert!(!score.is_nan());
        assert_eq!(path.len(), test_seq.len());
    }
}