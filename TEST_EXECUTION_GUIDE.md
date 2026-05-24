# Scanner System Test Execution Guide

This guide provides instructions for running the comprehensive test suite for the scanner system.

## Quick Start

### Running All Tests
```bash
# Run all tests using Makefile
make test-all

# Or run individual test types
make test-unit
make test-integration
make test-performance
make test-security
```

### Using the Integration Test Script
```bash
# Run comprehensive integration tests
cd scanners
chmod +x test_integration.sh
./test_integration.sh
```

## Test Overview

### Unit Tests
- **Location**: `scanners/*/tests/`
- **Coverage**: Individual component testing
- **Run**: `make test-unit` or `cargo test --lib`

### Integration Tests
- **Location**: `scanners/test_integration.sh`
- **Coverage**: End-to-end workflow testing
- **Run**: `make test-integration` or `./test_integration.sh`

### Performance Tests
- **Coverage**: Scalability and benchmarking
- **Run**: `make test-performance`

### Security Tests
- **Coverage**: Input validation and error handling
- **Run**: `make test-security`

## Test Prerequisites

### System Requirements
- Rust stable or beta
- Git
- jq (JSON processor)
- Bash (for integration tests)

### Installation
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install jq
sudo apt-get install jq  # Ubuntu/Debian
sudo yum install jq      # CentOS/RHEL

# Install Git
sudo apt-get install git  # Ubuntu/Debian
sudo yum install git      # CentOS/RHEL
```

### Verify Setup
```bash
make test-validate
```

## Detailed Test Execution

### 1. Unit Tests

#### CBOR Scanner Unit Tests
```bash
cd scanners/cbor_scanner
cargo test --lib -- --nocapture
```

#### Git Scanner Unit Tests
```bash
cd scanners/git_scanner
cargo test --lib -- --nocapture
```

#### Specific Test Categories
```bash
# HMM Model Tests
cargo test --lib -- hmm_tests -- --nocapture

# CBOR Processing Tests
cargo test --lib -- cbor_tests -- --nocapture

# Git Repository Tests
cargo test --lib -- git_tests -- --nocapture

# Language Detection Tests
cargo test --lib -- language_detection -- --nocapture
```

### 2. Integration Tests

#### Complete Workflow Test
```bash
cd scanners
./test_integration.sh
```

#### Individual Workflow Steps
```bash
# Build scanners
./scan_all.sh build

# Discover repositories
./scan_all.sh discover /path/to/repos /path/to/output.json

# Extract constants
./scan_all.sh extract /path/to/repos.json /path/to/output

# Train HMM
./scan_all.sh train /path/to/cbor/files /path/to/model.json 8

# Scan for anomalies
./scan_all.sh scan /path/to/cbor/files /path/to/model.json /path/to/output
```

### 3. Performance Tests

#### Scalability Testing
```bash
# Large file processing
cargo test --lib -- test_hmm_performance_large_sequence -- --nocapture

# Git discovery performance
cargo test --lib -- test_discover_git_repos_performance -- --nocapture

# Language detection performance
cargo test --lib -- test_language_detection_performance -- --nocapture
```

#### Benchmarking
```bash
# Generate benchmarks
cargo bench

# Run specific benchmarks
cargo test --lib -- bench_hmm_training -- --nocapture
```

### 4. Security Tests

#### Input Validation
```bash
# Path traversal protection
cargo test --lib -- test_path_traversal_protection -- --nocapture

# Command injection protection
cargo test --lib -- test_command_injection_protection -- --nocapture

# Error handling
cargo test --lib -- test_error_handling -- --nocapture
```

#### Security Scenarios
```bash
# Malformed input handling
./test_integration.sh

# Invalid configuration handling
cargo run --bin git_scanner -- scan -i . -c /non/existent/config.json -o /dev/null
```

## Test Configuration

### Environment Variables
```bash
# Enable verbose output
RUST_LOG=debug cargo test

# Set test timeout
CARGO_TEST_TIMEOUT=30s cargo test

# Use specific test runner
CARGO_TEST_THREADS=1 cargo test
```

### Custom Test Data
```bash
# Create custom test data directory
mkdir -p custom_test_data
export TEST_ROOT_DIR="custom_test_data"
./test_integration.sh
```

## Test Output

### Output Directories
```
test_integration_output/
├── cbor_analysis/
├── git_metadata/
├── scanner_output/
├── test_report.txt
└── *.json
```

### Understanding Results

#### JSON Output Files
- `test_hmm.json`: Trained HMM model
- `extracted_constants.json`: Extracted integer constants
- `discovered_repos.json`: Git repository metadata
- `languages.json`: Detected programming languages
- `anomalies.json`: Anomaly detection results

#### Test Report
```bash
# View test report
cat test_integration_output/test_report.txt

# Check test statistics
grep "File Statistics" test_integration_output/test_report.txt
```

### Success Criteria
- **Unit Tests**: 95%+ code coverage
- **Integration Tests**: All workflow steps successful
- **Performance Tests**: < 5 seconds for typical operations
- **Security Tests**: No vulnerabilities detected

## Troubleshooting

### Common Issues

#### Test Failures
```bash
# Check test logs
cargo test -- --nocapture

# Run specific failing test
cargo test --lib -- test_name -- --nocapture

# Clean and rebuild
make clean-test
make test-unit
```

#### Missing Dependencies
```bash
# Install missing tools
sudo apt-get install git jq

# Verify Rust installation
rustc --version
cargo --version
```

#### Permission Issues
```bash
# Fix executable permissions
chmod +x scanners/test_integration.sh
chmod +x scanners/scan_all.sh

# Fix file permissions
chmod 644 scanners/**/*.rs
```

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug ./test_integration.sh

# Run with verbose output
make test-verbose

# Generate detailed reports
make test-report
```

## Continuous Integration

### GitHub Actions
The test suite includes GitHub Actions workflows that run automatically:

- **Push/PR**: All tests on stable and beta
- **Weekly**: Full test suite on Monday at 2 AM
- **Matrix Testing**: Multiple Rust versions and test types

### Manual CI Trigger
```bash
# Trigger workflow manually
gh workflow run scanner_tests.yml
```

### CI Artifacts
After each run, artifacts are available:
- **Coverage Reports**: HTML coverage reports
- **Test Results**: JSON and log files
- **Final Report**: Summary of all test results

## Advanced Testing

### Custom Test Scenarios
```bash
# Create custom test data
mkdir -p custom_tests
export TEST_ROOT_DIR="custom_tests"

# Run with custom configuration
./test_integration.sh
```

### Parallel Testing
```bash
# Run tests in parallel
make test-parallel

# Use multiple test threads
CARGO_TEST_THREADS=4 cargo test
```

### Coverage Analysis
```bash
# Generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# View coverage
open target/tarpaulin/html/index.html
```

### Performance Profiling
```bash
# Install profiling tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin cbor_scanner -- --train -i test_data -o model.json
```

## Maintenance

### Updating Test Data
```bash
# Add new test CBOR files
echo -n -e "\x1a\x00\x00\x00\x65" > test_integration/cbor_samples/new_file.cbor

# Add new git repositories
cd test_integration/git_samples
git init new_repo
echo "test" > file.txt
git add .
git commit -m "test"
cd ../..
```

### Adding New Tests
```rust
// Add to existing test module
#[test]
fn test_new_feature() {
    // Test implementation
}
```

### Test Data Management
```bash
# Archive old test data
tar -czf test_data_archive_$(date +%Y%m%d).tar.gz test_integration/

# Clean old test data
find test_integration -name "*.tmp" -delete
```

## Best Practices

### Writing Tests
- Use descriptive test names
- Include setup/teardown when needed
- Test both success and failure cases
- Add performance benchmarks for critical functions

### Running Tests
- Always run tests before committing
- Use CI for comprehensive testing
- Monitor test execution times
- Keep test data organized

### Maintenance
- Update tests when code changes
- Remove obsolete test data
- Monitor test coverage
- Review test logs regularly

## Support

For issues or questions:
1. Check the troubleshooting section
2. Review test output logs
3. Check GitHub Actions artifacts
4. Create an issue with test details

## Summary

This comprehensive test suite ensures the reliability and performance of the scanner system. By following this guide, you can run tests effectively, understand results, and maintain test quality as the system evolves.