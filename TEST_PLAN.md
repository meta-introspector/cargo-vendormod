# Scanner System Test Plan

## Overview

This test plan outlines a comprehensive testing strategy for the scanner system, ensuring reliability, accuracy, and performance of both the CBOR scanner and Git scanner components.

## Test Strategy

### Testing Levels
1. **Unit Tests** - Individual component testing
2. **Integration Tests** - Component interaction testing
3. **End-to-End Tests** - Complete workflow validation
4. **Performance Tests** - Scalability and efficiency validation
5. **Security Tests** - Input validation and error handling

### Testing Principles
- **Comprehensive Coverage**: Test all major functionality and edge cases
- **Automated Execution**: All tests should be scriptable and repeatable
- **Clear Success Criteria**: Well-defined pass/fail conditions
- **Performance Baselines**: Establish performance metrics for regression detection

## Test Environment Setup

### Prerequisites
- Rust toolchain (stable version)
- Git installation
- CBOR test data sets
- Git repository test fixtures
- Sufficient disk space for performance testing

### Test Data Structure
```
test_data/
├── cbor_samples/
│   ├── valid_cbor/
│   │   ├── simple_integers.cbor
│   │   ├── complex_structures.cbor
│   │   └── large_file.cbor
│   ├── edge_cases/
│   │   ├── empty.cbor
│   │   ├── malformed.cbor
│   │   └── non_cbor_files/
│   └── performance/
│       ├── small_files/
│       ├── medium_files/
│       └── large_files/
├── git_repos/
│   ├── simple_repo/
│   ├── complex_repo/
│   ├── submodule_repo/
│   └── language_samples/
└── expected_outputs/
    ├── cbor_analysis/
    ├── git_metadata/
    └── anomaly_results/
```

## Test Cases

### 1. CBOR Scanner Unit Tests

#### 1.1 HMM Model Testing
```rust
#[cfg(test)]
mod hmm_tests {
    use super::*;
    
    #[test]
    fn test_hmm_initialization() {
        let hmm = HMM::new(4);
        assert_eq!(hmm.states.len(), 4);
        assert_eq!(hmm.state_names.len(), 4);
    }
    
    #[test]
    fn test_hmm_training() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3], vec![4, 5, 6]];
        hmm.train_simple(&sequences);
        
        // Check that emission probabilities sum to ~1.0 for each state
        for state in &hmm.states {
            let total: f64 = state.emissions.values().sum();
            assert!((total - 1.0).abs() < 0.1); // Allow some tolerance
        }
    }
    
    #[test]
    fn test_viterbi_scoring() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3], vec![4, 5, 6]];
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1, 2, 3];
        let (score, path) = hmm.viterbi_score(&test_seq);
        assert!(!score.is_nan());
        assert_eq!(path.len(), test_seq.len());
    }
}
```

#### 1.2 CBOR File Processing Tests
```rust
#[cfg(test)]
mod cbor_tests {
    use super::*;
    
    #[test]
    fn test_cbor_parsing() {
        let test_data = vec![0x1a, 0x00, 0x00, 0x00, 0x64]; // CBOR uint(100)
        let analysis = scan_cbor_file_from_bytes(&test_data);
        assert!(!analysis.integers.is_empty());
        assert!(analysis.integers.contains(&100));
    }
    
    #[test]
    fn test_integer_extraction() {
        let test_data = vec![0x1a, 0x00, 0x00, 0x00, 0x64, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64];
        let analysis = scan_cbor_file_from_bytes(&test_data);
        assert_eq!(analysis.integers.len(), 2);
    }
    
    #[test]
    fn test_malformed_cbor_handling() {
        let malformed_data = vec![0xff, 0x00, 0x00]; // Invalid CBOR
        let result = scan_cbor_file_from_bytes(&malformed_data);
        // Should not panic, should handle gracefully
        assert!(result.is_ok());
    }
}
```

#### 1.3 Anomaly Detection Tests
```rust
#[cfg(test)]
mod anomaly_tests {
    use super::*;
    
    #[test]
    fn test_anomaly_detection() {
        let mut hmm = HMM::new(2);
        let normal_sequences = vec![vec![1, 2, 3, 4, 5], vec![10, 20, 30, 40, 50]];
        hmm.train_simple(&normal_sequences);
        
        let anomalous_seq = vec![1, 2, 255, 4, 5]; // Contains outlier byte
        let anomalies = detect_anomalies(&anomalous_seq, &hmm, &vec![0, 0, 1, 0, 0]);
        assert!(!anomalies.is_empty());
    }
    
    #[test]
    fn test_rare_transition_detection() {
        let mut hmm = HMM::new(2);
        let sequences = vec![vec![1, 2, 3], vec![1, 2, 3]];
        hmm.train_simple(&sequences);
        
        let test_seq = vec![1, 255, 3]; // Rare transition
        let anomalies = detect_anomalies(&test_seq, &hmm, &vec![0, 0, 1]);
        assert!(anomalies.len() > 0);
    }
}
```

### 2. Git Scanner Unit Tests

#### 2.1 Repository Discovery Tests
```rust
#[cfg(test)]
mod git_tests {
    use super::*;
    
    #[test]
    fn test_git_repo_detection() {
        // Create temporary git repository
        let temp_dir = tempfile::tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        
        let repos = discover_git_repos(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].path, temp_dir.path());
    }
    
    #[test]
    fn test_submodule_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let git_file = temp_dir.path().join(".git");
        let gitdir_content = "gitdir: ../.git/modules/test";
        fs::write(&git_file, gitdir_content).unwrap();
        
        let repo = scan_git_repo(temp_dir.path()).unwrap();
        assert!(repo.is_submodule);
    }
    
    #[test]
    fn test_language_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rust_file = temp_dir.path().join("src/main.rs");
        fs::write(&rust_file, "fn main() {}").unwrap();
        
        let languages = detect_languages(temp_dir.path()).unwrap();
        assert!(languages.contains("Rust"));
    }
}
```

#### 2.2 Scanner Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_scanner_execution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.cbor");
        fs::write(&test_file, vec![0x1a, 0x00, 0x00, 0x00, 0x64]).unwrap();
        
        let config = ScannerConfig {
            name: "test_scanner".to_string(),
            extensions: vec!["cbor".to_string()],
            command: "echo 'test'".to_string(),
            output_dir: "test_output".to_string(),
        };
        
        let result = run_scanner(temp_dir.path(), &config, temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }
}
```

### 3. Integration Tests

#### 3.1 End-to-End Workflow Tests
```bash
#!/bin/bash
# test_full_workflow.sh

set -euo pipefail

# Setup test environment
setup_test_env() {
    echo "Setting up test environment..."
    mkdir -p test_output
    mkdir -p test_data/cbor_samples
    mkdir -p test_data/git_repos
    
    # Create test CBOR files
    echo -n -e "\x1a\x00\x00\x00\x64" > test_data/cbor_samples/test1.cbor
    echo -n -e "\x1b\x00\x00\x00\x00\x00\x00\x00\x64" > test_data/cbor_samples/test2.cbor
    
    # Create test git repository
    cd test_data/git_repos
    git init simple_repo
    cd simple_repo
    echo "fn main() {}" > src/main.rs
    git add .
    git commit -m "Initial commit"
    cd ../..
}

# Test CBOR scanner functionality
test_cbor_scanner() {
    echo "Testing CBOR scanner..."
    
    # Test training
    cargo run --bin cbor_scanner -- train \
        -i test_data/cbor_samples \
        -o test_output/test_model.json \
        -s 4
    
    # Test scanning
    cargo run --bin cbor_scanner -- scan \
        -i test_data/cbor_samples \
        -m test_output/test_model.json \
        -o test_output/cbor_analysis
    
    # Test extraction
    cargo run --bin cbor_scanner -- extract \
        -i test_data/cbor_samples \
        -o test_output/extracted_constants.json
    
    # Verify outputs
    if [ -f test_output/test_model.json ]; then
        echo "✓ HMM model created successfully"
    else
        echo "✗ HMM model creation failed"
        exit 1
    fi
    
    if [ -f test_output/extracted_constants.json ]; then
        echo "✓ Constants extracted successfully"
    else
        echo "✗ Constants extraction failed"
        exit 1
    fi
}

# Test git scanner functionality
test_git_scanner() {
    echo "Testing Git scanner..."
    
    # Test discovery
    cargo run --bin git_scanner -- discover \
        -i test_data/git_repos \
        -o test_output/discovered_repos.json \
        --recursive
    
    # Test language detection
    cargo run --bin git_scanner -- list-languages \
        -i test_data/git_repos \
        -o test_output/languages.json
    
    # Verify outputs
    if [ -f test_output/discovered_repos.json ]; then
        echo "✓ Repositories discovered successfully"
    else
        echo "✗ Repository discovery failed"
        exit 1
    fi
}

# Test complete workflow
test_complete_workflow() {
    echo "Testing complete workflow..."
    
    ./scan_all.sh build
    ./scan_all.sh discover test_data/git_repos test_output/repos.json
    ./scan_all.sh extract test_output/repos.json test_output/analysis
    ./scan_all.sh train test_data/cbor_samples test_output/hmm.json 4
    ./scan_all.sh scan test_data/cbor_samples test_output/hmm.json test_output/anomalies
    
    echo "✓ Complete workflow executed successfully"
}

# Cleanup
cleanup() {
    echo "Cleaning up test environment..."
    rm -rf test_output
    rm -rf test_data
}

# Main test execution
main() {
    setup_test_env
    test_cbor_scanner
    test_git_scanner
    test_complete_workflow
    cleanup
    echo "✓ All tests passed!"
}

main "$@"
```

### 4. Performance Tests

#### 4.1 Scalability Testing
```bash
#!/bin/bash
# test_performance.sh

set -euo pipefail

# Test with different file sizes
test_scalability() {
    echo "Testing scalability..."
    
    # Generate test files of different sizes
    for size in 1K 10K 100K 1M 10M; do
        echo "Testing with $size files..."
        
        # Generate test data
        dd if=/dev/urandom of=test_data/performance/test_${size}.cbor bs=$size count=1
        
        # Time the operations
        start_time=$(date +%s.%N)
        
        cargo run --bin cbor_scanner -- extract \
            -i test_data/performance \
            -o test_output/performance_${size}
        
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc)
        
        echo "Processing $size files took $duration seconds"
        
        # Record performance metrics
        echo "$size,$duration" >> test_output/performance_metrics.csv
    done
}

# Memory usage testing
test_memory_usage() {
    echo "Testing memory usage..."
    
    # Use valgrind or similar tool if available
    if command -v valgrind &> /dev/null; then
        valgrind --tool=massif \
            cargo run --bin cbor_scanner -- extract \
            -i test_data/performance \
            -o test_output/memory_test
    else
        echo "Valgrind not available, skipping memory analysis"
    fi
}

# Concurrency testing
test_concurrency() {
    echo "Testing concurrency..."
    
    # Process multiple files in parallel
    start_time=$(date +%s.%N)
    
    for i in {1..10}; do
        cargo run --bin cbor_scanner -- extract \
            -i test_data/performance \
            -o test_output/concurrent_$i &
    done
    
    wait
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    echo "Concurrent processing took $duration seconds"
}
```

### 5. Security Tests

#### 5.1 Input Validation Tests
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_protection() {
        let malicious_path = "../../../etc/passwd";
        let result = scan_cbor_file(Path::new(malicious_path));
        // Should handle gracefully without exposing sensitive files
        assert!(result.is_err() || result.unwrap().path != malicious_path);
    }
    
    #[test]
    fn test_command_injection_protection() {
        let config = ScannerConfig {
            name: "test".to_string(),
            extensions: vec!["txt".to_string()],
            command: "echo {malicious}".to_string(),
            output_dir: "output".to_string(),
        };
        
        let result = run_scanner(Path::new("test"), &config, Path::new("test"));
        // Should not execute malicious commands
        assert!(result.is_ok());
    }
}
```

#### 5.2 Error Handling Tests
```bash
#!/bin/bash
# test_error_handling.sh

test_missing_files() {
    echo "Testing error handling for missing files..."
    
    # Test with non-existent directory
    if cargo run --bin cbor_scanner -- extract \
        -i /non/existent/path \
        -o test_output/error_test 2>/dev/null; then
        echo "✗ Should have failed with missing directory"
        exit 1
    else
        echo "✓ Correctly handled missing directory"
    fi
}

test_invalid_cbor() {
    echo "Testing error handling for invalid CBOR..."
    
    # Create invalid CBOR file
    echo "invalid cbor content" > test_data/invalid.cbor
    
    if cargo run --bin cbor_scanner -- extract \
        -i test_data/invalid.cbor \
        -o test_output/invalid_test 2>/dev/null; then
        echo "✗ Should have handled invalid CBOR gracefully"
        exit 1
    else
        echo "✓ Correctly handled invalid CBOR"
    fi
}
```

## Test Execution Framework

### Test Automation
```makefile
# Makefile for test execution

.PHONY: test test-unit test-integration test-performance test-security test-all

# Run all tests
test-all: test-unit test-integration test-performance test-security

# Unit tests
test-unit:
	cargo test --lib -- --nocapture

# Integration tests
test-integration:
	bash scripts/test_integration.sh

# Performance tests
test-performance:
	bash scripts/test_performance.sh

# Security tests
test-security:
	bash scripts/test_security.sh

# Clean test artifacts
clean-test:
	rm -rf test_output/
	rm -rf test_data/
```

### Continuous Integration
```yaml
# .github/workflows/test.yml
name: Scanner Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust-version: [stable, beta]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust-version }}
    
    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y git
    
    - name: Run unit tests
      run: make test-unit
    
    - name: Run integration tests
      run: make test-integration
    
    - name: Run performance tests
      run: make test-performance
    
    - name: Run security tests
      run: make test-security
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: test_output/
```

## Test Metrics and Reporting

### Success Criteria
- **Unit Tests**: 95%+ code coverage
- **Integration Tests**: 100% critical path coverage
- **Performance Tests**: < 1 second for small files, < 30 seconds for large files
- **Security Tests**: 100% vulnerability coverage

### Reporting Format
```json
{
  "test_results": {
    "unit_tests": {
      "total": 45,
      "passed": 45,
      "failed": 0,
      "coverage": "95.2%"
    },
    "integration_tests": {
      "total": 12,
      "passed": 12,
      "failed": 0,
      "critical_path_coverage": "100%"
    },
    "performance_tests": {
      "small_files": "0.8s",
      "medium_files": "5.2s",
      "large_files": "28.7s",
      "memory_usage": "128MB"
    },
    "security_tests": {
      "vulnerabilities_found": 0,
      "tests_passed": 15,
      "tests_failed": 0
    }
  },
  "recommendations": [
    "Consider adding more edge case tests",
    "Optimize memory usage for large files",
    "Add more comprehensive error handling"
  ]
}
```

## Test Maintenance

### Test Data Management
- Version control test data in separate repository
- Regularly update test data to reflect real-world scenarios
- Archive old test data for regression testing

### Test Review Process
- Regular test code reviews
- Performance benchmark comparisons
- Security audit of test infrastructure
- Documentation updates for new tests

### Continuous Improvement
- Monitor test execution times and optimize slow tests
- Add new test cases based on bug reports
- Update test coverage targets as codebase evolves
- Regular test suite refactoring to maintain maintainability

## Conclusion

This comprehensive test plan ensures thorough validation of the scanner system across all critical dimensions. The automated testing framework provides continuous quality assurance while the detailed test cases cover both expected behavior and edge cases. Regular execution of these tests will maintain high code quality and system reliability as the scanner system evolves.