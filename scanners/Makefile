# Makefile for Scanner System Testing

.PHONY: test test-unit test-integration test-performance test-security test-all clean-test run-tests

# Default test target
test: test-all

# Run all tests
test-all: test-unit test-integration test-performance test-security

# Unit tests
test-unit:
	@echo "Running unit tests..."
	@cd scanners/cbor_scanner && cargo test --lib -- --nocapture --test-threads=1
	@cd scanners/git_scanner && cargo test --lib -- --nocapture --test-threads=1
	@echo "Unit tests completed"

# Integration tests
test-integration:
	@echo "Running integration tests..."
	@chmod +x scanners/test_integration.sh
	@cd scanners && ./test_integration.sh
	@echo "Integration tests completed"

# Performance tests
test-performance:
	@echo "Running performance tests..."
	@cd scanners && ./test_integration.sh
	@echo "Performance tests completed"

# Security tests
test-security:
	@echo "Running security tests..."
	@cd scanners && ./test_integration.sh
	@echo "Security tests completed"

# Clean test artifacts
clean-test:
	@echo "Cleaning test artifacts..."
	@rm -rf test_integration/
	@rm -rf test_integration_output/
	@rm -rf scanners/target/
	@rm -rf scanners/*/target/
	@echo "Test artifacts cleaned"

# Run tests with verbose output
test-verbose:
	@echo "Running tests with verbose output..."
	@cd scanners/cbor_scanner && cargo test --lib -- --nocapture --test-threads=1 -- --verbose
	@cd scanners/git_scanner && cargo test --lib -- --nocapture --test-threads=1 -- --verbose
	@cd scanners && ./test_integration.sh

# Run specific test
test-specific:
	@echo "Running specific test..."
	@cd scanners/cbor_scanner && cargo test --lib -- --exact --test-threads=1 $(filter-out $@,$(MAKECMDGOALS))
	@cd scanners/git_scanner && cargo test --lib -- --exact --test-threads=1 $(filter-out $@,$(MAKECMDGOALS))

# Run tests with coverage
test-coverage:
	@echo "Running tests with coverage..."
	@cd scanners/cbor_scanner && cargo install cargo-tarpaulin && cargo tarpaulin --out html
	@cd scanners/git_scanner && cargo install cargo-tarpaulin && cargo tarpaulin --out html
	@echo "Coverage reports generated in target/tarpaulin/html/"

# Run tests in parallel
test-parallel:
	@echo "Running tests in parallel..."
	@cd scanners/cbor_scanner && cargo test --lib -- --test-threads=4
	@cd scanners/git_scanner && cargo test --lib -- --test-threads=4 &
	@wait
	@cd scanners && ./test_integration.sh
	@echo "Parallel tests completed"

# Run tests with specific features
test-features:
	@echo "Running tests with features..."
	@cd scanners/cbor_scanner && cargo test --lib --features serde_json -- --nocapture
	@cd scanners/git_scanner && cargo test --lib --features rayon -- --nocapture
	@echo "Feature-specific tests completed"

# Run tests and generate report
test-report:
	@echo "Running tests and generating report..."
	@mkdir -p test_reports
	@cd scanners/cbor_scanner && cargo test --lib -- --nocapture 2>&1 | tee ../../test_reports/cbor_scanner_tests.log
	@cd scanners/git_scanner && cargo test --lib -- --nocapture 2>&1 | tee ../../test_reports/git_scanner_tests.log
	@cd scanners && ./test_integration.sh 2>&1 | tee ../../test_reports/integration_tests.log
	@echo "Test reports generated in test_reports/"

# Quick smoke test
test-smoke:
	@echo "Running smoke tests..."
	@cd scanners && cargo build --release
	@cd scanners && echo "test" | cargo run --bin cbor_scanner -- extract -i /dev/stdin -o /dev/null
	@cd scanners && cargo run --bin git_scanner -- discover -i . -o /dev/null
	@echo "Smoke tests completed"

# Validate test setup
test-validate:
	@echo "Validating test setup..."
	@which cargo > /dev/null || (echo "Cargo not found" && exit 1)
	@which jq > /dev/null || (echo "jq not found" && exit 1)
	@which git > /dev/null || (echo "git not found" && exit 1)
	@echo "Test setup validated"

# Test individual components
test-cbor-scanner:
	@echo "Testing CBOR Scanner..."
	@cd scanners/cbor_scanner && cargo test --lib -- --nocapture

test-git-scanner:
	@echo "Testing Git Scanner..."
	@cd scanners/git_scanner && cargo test --lib -- --nocapture

test-workflow:
	@echo "Testing Workflow..."
	@cd scanners && ./test_integration.sh

# Test error conditions
test-error:
	@echo "Testing error conditions..."
	@cd scanners/cbor_scanner && cargo test --lib -- test_error_handling
	@cd scanners/git_scanner && cargo test --lib -- test_error_handling

# Test performance benchmarks
test-bench:
	@echo "Running performance benchmarks..."
	@cd scanners/cbor_scanner && cargo bench
	@cd scanners/git_scanner && cargo bench

# Test with minimal dependencies
test-minimal:
	@echo "Running tests with minimal dependencies..."
	@cd scanners/cbor_scanner && cargo test --no-default-features
	@cd scanners/git_scanner && cargo test --no-default-features

# Test with all features
test-full:
	@echo "Running tests with all features..."
	@cd scanners/cbor_scanner && cargo test --all-features
	@cd scanners/git_scanner && cargo test --all-features

# Test specific scenarios
test-scenarios:
	@echo "Running specific test scenarios..."
	@cd scanners/cbor_scanner && cargo test --lib -- exact -- test_hmm_initialization
	@cd scanners/cbor_scanner && cargo test --lib -- exact -- test_cbor_parsing_valid
	@cd scanners/git_scanner && cargo test --lib -- exact -- test_discover_git_repos_single_repo
	@cd scanners/git_scanner && cargo test --lib -- exact -- test_detect_languages_rust

# Test output validation
test-output:
	@echo "Validating test outputs..."
	@find test_integration_output -name "*.json" | while read file; do \
		if jq empty "$file" 2>/dev/null; then \
			echo "Valid JSON: $$file"; \
		else \
			echo "Invalid JSON: $$file"; \
		fi; \
	done
	@echo "Output validation completed"

# Test cleanup and validation
test-clean-and-validate:
	@echo "Cleaning and validating..."
	@$(MAKE) clean-test
	@$(MAKE) test-validate
	@echo "Cleanup and validation completed"