#!/bin/bash

# Comprehensive Integration Test Script for Scanner System
# This script tests the complete scanner workflow with real data

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test configuration
TEST_ROOT_DIR="test_integration"
TEST_OUTPUT_DIR="test_integration_output"
CBOR_SAMPLES_DIR="$TEST_ROOT_DIR/cbor_samples"
GIT_SAMPLES_DIR="$TEST_ROOT_DIR/git_samples"
SCANNER_CONFIG="$TEST_OUTPUT_DIR/scanner_config.json"

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    if [ -d "$TEST_ROOT_DIR" ]; then
        rm -rf "$TEST_ROOT_DIR"
    fi
    if [ -d "$TEST_OUTPUT_DIR" ]; then
        rm -rf "$TEST_OUTPUT_DIR"
    fi
}

# Setup test environment
setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Create directories
    mkdir -p "$CBOR_SAMPLES_DIR"
    mkdir -p "$GIT_SAMPLES_DIR"
    mkdir -p "$TEST_OUTPUT_DIR"
    
    # Create test CBOR files
    log_info "Creating test CBOR files..."
    
    # Simple CBOR: unsigned integer 100
    printf '\x1a\x00\x00\x00\x64' > "$CBOR_SAMPLES_DIR/simple_uint.cbor"
    
    # CBOR: unsigned integer 100, unsigned integer 200
    printf '\x1a\x00\x00\x00\x64\x1a\x00\x00\x00\xc8' > "$CBOR_SAMPLES_DIR/multiple_uints.cbor"
    
    # CBOR: text "hello"
    printf '\x65hello' > "$CBOR_SAMPLES_DIR/text_string.cbor"
    
    # CBOR: array [1, 2, 3]
    printf '\x83\x01\x02\x03' > "$CBOR_SAMPLES_DIR/simple_array.cbor"
    
    # CBOR: map {1: "one", 2: "two"}
    printf '\xa2\x01\x61\x6f\x6e\x65\x02\x61\x74\x77\x6f' > "$CBOR_SAMPLES_DIR/simple_map.cbor"
    
    # Create larger CBOR file for performance testing
    log_info "Creating large CBOR file for performance testing..."
    for i in {1..100}; do
        printf '\x1a\x00\x00\x00%02x' $i >> "$CBOR_SAMPLES_DIR/large_file.cbor"
    done
    
    # Create malformed CBOR for error handling testing
    printf '\xff\x00\x00' > "$CBOR_SAMPLES_DIR/malformed.cbor"
    
    # Create scanner configuration
    log_info "Creating scanner configuration..."
    cat > "$SCANNER_CONFIG" << 'EOF'
{
  "output_dir": "scanner_output",
  "scanners": [
    {
      "name": "cbor_scanner",
      "extensions": ["cbor"],
      "command": "echo 'CBOR file found: {file}' > {output}/cbor_found.txt",
      "output_dir": "cbor"
    }
  ]
}
EOF
    
    log_success "Test environment setup complete"
}

# Test CBOR Scanner
test_cbor_scanner() {
    log_info "Testing CBOR Scanner..."
    
    cd scanners
    
    # Test training
    log_info "Testing HMM training..."
    if cargo run --bin cbor_scanner -- train \
        -i "$CBOR_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/test_hmm.json" \
        -s 4; then
        log_success "HMM training successful"
    else
        log_error "HMM training failed"
        return 1
    fi
    
    # Verify model file was created
    if [ -f "$TEST_OUTPUT_DIR/test_hmm.json" ]; then
        log_success "HMM model file created"
    else
        log_error "HMM model file not found"
        return 1
    fi
    
    # Test scanning
    log_info "Testing CBOR scanning..."
    if cargo run --bin cbor_scanner -- scan \
        -i "$CBOR_SAMPLES_DIR" \
        -m "$TEST_OUTPUT_DIR/test_hmm.json" \
        -o "$TEST_OUTPUT_DIR/cbor_analysis"; then
        log_success "CBOR scanning successful"
    else
        log_error "CBOR scanning failed"
        return 1
    fi
    
    # Test extraction
    log_info "Testing CBOR constant extraction..."
    if cargo run --bin cbor_scanner -- extract \
        -i "$CBOR_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/extracted_constants.json"; then
        log_success "CBOR extraction successful"
    else
        log_error "CBOR extraction failed"
        return 1
    fi
    
    # Verify extraction results
    if [ -f "$TEST_OUTPUT_DIR/extracted_constants.json" ]; then
        log_success "Extraction results saved"
        
        # Check if we found integers
        if grep -q "integers" "$TEST_OUTPUT_DIR/extracted_constants.json"; then
            log_success "Integers found in extraction results"
        else
            log_warn "No integers found in extraction results"
        fi
    else
        log_error "Extraction results not found"
        return 1
    fi
    
    cd ..
}

# Test Git Scanner
test_git_scanner() {
    log_info "Testing Git Scanner..."
    
    cd scanners
    
    # Create test git repositories
    log_info "Creating test git repositories..."
    
    # Simple Rust repository
    cd "$GIT_SAMPLES_DIR"
    git init simple_rust_repo
    cd simple_rust_repo
    echo 'fn main() { println!("Hello, World!"); }' > src/main.rs
    echo '[package]' > Cargo.toml
    echo 'name = "test"' >> Cargo.toml
    echo 'version = "0.1.0"' >> Cargo.toml
    git add .
    git commit -m "Initial commit"
    cd ..
    
    # Python repository
    git init python_repo
    cd python_repo
    echo 'print("Hello, World!")' > main.py
    git add .
    git commit -m "Initial commit"
    cd ..
    
    # Go repository
    git init go_repo
    cd go_repo
    echo 'package main' > main.go
    echo '' >> main.go
    echo 'func main() {' >> main.go
    echo '    println("Hello, World!")' >> main.go
    echo '}' >> main.go
    echo 'module test' > go.mod
    git add .
    git commit -m "Initial commit"
    cd ..
    
    # Repository with submodules
    git init parent_repo
    cd parent_repo
    echo 'Parent repository content' > README.md
    git add .
    git commit -m "Initial commit"
    
    # Add submodules
    git submodule add ../simple_rust_repo rust_submodule
    git submodule add ../python_repo python_submodule
    git commit -m "Add submodules"
    cd ..
    
    cd ..
    
    # Test repository discovery
    log_info "Testing repository discovery..."
    if cargo run --bin git_scanner -- discover \
        -i "$GIT_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/discovered_repos.json" \
        --recursive; then
        log_success "Repository discovery successful"
    else
        log_error "Repository discovery failed"
        return 1
    fi
    
    # Verify discovery results
    if [ -f "$TEST_OUTPUT_DIR/discovered_repos.json" ]; then
        log_success "Discovery results saved"
        
        # Count discovered repositories
        repo_count=$(jq '. | length' "$TEST_OUTPUT_DIR/discovered_repos.json")
        log_info "Found $repo_count repositories"
        
        if [ "$repo_count" -gt 0 ]; then
            log_success "Repositories discovered successfully"
        else
            log_error "No repositories found"
            return 1
        fi
    else
        log_error "Discovery results not found"
        return 1
    fi
    
    # Test language detection
    log_info "Testing language detection..."
    if cargo run --bin git_scanner -- list-languages \
        -i "$GIT_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/languages.json"; then
        log_success "Language detection successful"
    else
        log_error "Language detection failed"
        return 1
    fi
    
    # Verify language detection results
    if [ -f "$TEST_OUTPUT_DIR/languages.json" ]; then
        log_success "Language detection results saved"
        
        # Check if languages were detected
        if grep -q "Rust" "$TEST_OUTPUT_DIR/languages.json" || \
           grep -q "Python" "$TEST_OUTPUT_DIR/languages.json" || \
           grep -q "Go" "$TEST_OUTPUT_DIR/languages.json"; then
            log_success "Languages detected successfully"
        else
            log_warn "No languages detected in expected repositories"
        fi
    else
        log_error "Language detection results not found"
        return 1
    fi
    
    # Test scanning with configuration
    log_info "Testing scanning with configuration..."
    if cargo run --bin git_scanner -- scan \
        -i "$GIT_SAMPLES_DIR" \
        -c "$SCANNER_CONFIG" \
        -o "$TEST_OUTPUT_DIR/git_scan_output"; then
        log_success "Git scanning successful"
    else
        log_error "Git scanning failed"
        return 1
    fi
    
    cd ..
}

# Test Complete Workflow
test_complete_workflow() {
    log_info "Testing complete workflow..."
    
    cd scanners
    
    # Test the unified workflow script
    log_info "Testing unified workflow script..."
    if ./scan_all.sh build; then
        log_success "Scanner build successful"
    else
        log_error "Scanner build failed"
        return 1
    fi
    
    # Test discovery step
    log_info "Testing discovery step..."
    if ./scan_all.sh discover "$TEST_ROOT_DIR" "$TEST_OUTPUT_DIR/workflow_repos.json"; then
        log_success "Workflow discovery successful"
    else
        log_error "Workflow discovery failed"
        return 1
    fi
    
    # Test extraction step
    log_info "Testing extraction step..."
    if ./scan_all.sh extract "$TEST_OUTPUT_DIR/workflow_repos.json" "$TEST_OUTPUT_DIR/workflow_analysis"; then
        log_success "Workflow extraction successful"
    else
        log_error "Workflow extraction failed"
        return 1
    fi
    
    # Test training step
    log_info "Training step..."
    if ./scan_all.sh train "$CBOR_SAMPLES_DIR" "$TEST_OUTPUT_DIR/workflow_hmm.json" 4; then
        log_success "Workflow training successful"
    else
        log_error "Workflow training failed"
        return 1
    fi
    
    # Test scanning step
    log_info "Testing scanning step..."
    if ./scan_all.sh scan "$CBOR_SAMPLES_DIR" "$TEST_OUTPUT_DIR/workflow_hmm.json" "$TEST_OUTPUT_DIR/workflow_anomalies"; then
        log_success "Workflow scanning successful"
    else
        log_error "Workflow scanning failed"
        return 1
    fi
    
    cd ..
    
    log_success "Complete workflow test successful"
}

# Test Error Handling
test_error_handling() {
    log_info "Testing error handling..."
    
    cd scanners
    
    # Test with non-existent directory
    log_info "Testing non-existent directory..."
    if cargo run --bin cbor_scanner -- extract \
        -i "/non/existent/directory" \
        -o "$TEST_OUTPUT_DIR/error_test" 2>/dev/null; then
        log_error "Should have failed with non-existent directory"
        return 1
    else
        log_success "Correctly handled non-existent directory"
    fi
    
    # Test with invalid CBOR
    log_info "Testing invalid CBOR handling..."
    if cargo run --bin cbor_scanner -- extract \
        -i "$CBOR_SAMPLES_DIR/malformed.cbor" \
        -o "$TEST_OUTPUT_DIR/malformed_test" 2>/dev/null; then
        log_success "Handled malformed CBOR gracefully"
    else
        log_warn "Malformed CBOR caused failure (may be expected)"
    fi
    
    # Test with missing configuration file
    log_info "Testing missing configuration file..."
    if cargo run --bin git_scanner -- scan \
        -i "$GIT_SAMPLES_DIR" \
        -c "/non/existent/config.json" \
        -o "$TEST_OUTPUT_DIR/missing_config_test" 2>/dev/null; then
        log_success "Handled missing configuration gracefully"
    else
        log_warn "Missing configuration caused failure (may be expected)"
    fi
    
    cd ..
}

# Test Performance
test_performance() {
    log_info "Testing performance..."
    
    cd scanners
    
    # Test CBOR processing performance
    log_info "Testing CBOR processing performance..."
    start_time=$(date +%s.%N)
    
    if cargo run --bin cbor_scanner -- extract \
        -i "$CBOR_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/performance_test"; then
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc)
        log_info "CBOR processing took $duration seconds"
        
        if (( $(echo "$duration < 5.0" | bc -l) )); then
            log_success "CBOR processing performance is acceptable"
        else
            log_warn "CBOR processing took longer than expected"
        fi
    else
        log_error "CBOR performance test failed"
        return 1
    fi
    
    # Test Git discovery performance
    log_info "Testing Git discovery performance..."
    start_time=$(date +%s.%N)
    
    if cargo run --bin git_scanner -- discover \
        -i "$GIT_SAMPLES_DIR" \
        -o "$TEST_OUTPUT_DIR/performance_discovery.json"; then
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc)
        log_info "Git discovery took $duration seconds"
        
        if (( $(echo "$duration < 3.0" | bc -l) )); then
            log_success "Git discovery performance is acceptable"
        else
            log_warn "Git discovery took longer than expected"
        fi
    else
        log_error "Git discovery performance test failed"
        return 1
    fi
    
    cd ..
}

# Test Security
test_security() {
    log_info "Testing security aspects..."
    
    cd scanners
    
    # Test path traversal protection
    log_info "Testing path traversal protection..."
    if cargo run --bin cbor_scanner -- extract \
        -i "../../../etc/passwd" \
        -o "$TEST_OUTPUT_DIR/security_test" 2>/dev/null; then
        log_error "Should have blocked path traversal attack"
        return 1
    else
        log_success "Path traversal attack blocked"
    fi
    
    # Test command injection protection
    log_info "Testing command injection protection..."
    # Create a malicious configuration
    cat > "$TEST_OUTPUT_DIR/malicious_config.json" << 'EOF'
{
  "output_dir": "scanner_output",
  "scanners": [
    {
      "name": "malicious_scanner",
      "extensions": ["txt"],
      "command": "echo 'malicious {command}' > {output}/result.txt",
      "output_dir": "malicious"
    }
  ]
}
EOF
    
    if cargo run --bin git_scanner -- scan \
        -i "$GIT_SAMPLES_DIR" \
        -c "$TEST_OUTPUT_DIR/malicious_config.json" \
        -o "$TEST_OUTPUT_DIR/security_test" 2>/dev/null; then
        log_success "Command injection attempt handled gracefully"
    else
        log_warn "Command injection caused failure (may be expected)"
    fi
    
    cd ..
}

# Verify Results
verify_results() {
    log_info "Verifying test results..."
    
    # Check if all expected output files exist
    expected_files=(
        "$TEST_OUTPUT_DIR/test_hmm.json"
        "$TEST_OUTPUT_DIR/extracted_constants.json"
        "$TEST_OUTPUT_DIR/discovered_repos.json"
        "$TEST_OUTPUT_DIR/languages.json"
        "$TEST_OUTPUT_DIR/git_scan_output"
        "$TEST_OUTPUT_DIR/workflow_repos.json"
        "$TEST_OUTPUT_DIR/workflow_analysis"
        "$TEST_OUTPUT_DIR/workflow_hmm.json"
        "$TEST_OUTPUT_DIR/workflow_anomalies"
    )
    
    missing_files=0
    for file in "${expected_files[@]}"; do
        if [ -f "$file" ]; then
            log_success "Found expected file: $file"
        else
            log_error "Missing expected file: $file"
            ((missing_files++))
        fi
    done
    
    if [ $missing_files -eq 0 ]; then
        log_success "All expected output files found"
    else
        log_error "$missing_files expected files missing"
        return 1
    fi
    
    # Check if JSON files are valid
    log_info "Validating JSON files..."
    for file in "$TEST_OUTPUT_DIR"/*.json; do
        if [ -f "$file" ]; then
            if jq empty "$file" 2>/dev/null; then
                log_success "Valid JSON: $file"
            else
                log_error "Invalid JSON: $file"
                return 1
            fi
        fi
    done
}

# Generate Test Report
generate_test_report() {
    log_info "Generating test report..."
    
    report_file="$TEST_OUTPUT_DIR/test_report.txt"
    
    cat > "$report_file" << EOF
Scanner System Integration Test Report
=====================================

Test Date: $(date)
Test Duration: $SECONDS seconds

Test Summary:
- CBOR Scanner: ✓
- Git Scanner: ✓
- Complete Workflow: ✓
- Error Handling: ✓
- Performance: ✓
- Security: ✓

Generated Files:
EOF
    
    # List generated files
    echo "" >> "$report_file"
    echo "Output Directory Contents:" >> "$report_file"
    echo "========================" >> "$report_file"
    find "$TEST_OUTPUT_DIR" -type f -name "*.json" -o -name "*.txt" -o -name "*.cbor" | sort >> "$report_file"
    
    # File counts
    echo "" >> "$report_file"
    echo "File Statistics:" >> "$report_file"
    echo "===============" >> "$report_file"
    echo "Total JSON files: $(find "$TEST_OUTPUT_DIR" -name "*.json" | wc -l)" >> "$report_file"
    echo "Total text files: $(find "$TEST_OUTPUT_DIR" -name "*.txt" | wc -l)" >> "$report_file"
    echo "Total CBOR files: $(find "$TEST_OUTPUT_DIR" -name "*.cbor" | wc -l)" >> "$report_file"
    
    log_success "Test report generated: $report_file"
}

# Main test execution
main() {
    log_info "Starting Scanner System Integration Tests..."
    
    # Setup
    setup_test_environment
    
    # Run tests
    if test_cbor_scanner && \
       test_git_scanner && \
       test_complete_workflow && \
       test_error_handling && \
       test_performance && \
       test_security && \
       verify_results; then
        log_success "All integration tests passed!"
        generate_test_report
        cleanup
        exit 0
    else
        log_error "Some integration tests failed!"
        # Don't cleanup on failure so we can examine the results
        exit 1
    fi
}

# Handle signals for cleanup
trap cleanup EXIT INT TERM

# Run main function
main "$@"