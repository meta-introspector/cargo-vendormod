#!/usr/bin/env bash
# Workload Performance Test Runner for cargo-vendormod
# Measures test execution time, memory usage, and coverage

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
cd "$PROJECT_ROOT/tools/cargo-vendormod"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}  Cargo-Vendormod Workload Test Suite${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Create output directory
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="$PROJECT_ROOT/test_results/workload_$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Function to measure test execution
run_test_workload() {
    local test_name=$1
    local test_cmd=$2
    
    echo -e "${YELLOW}Running: $test_name${NC}"
    echo "Command: $test_cmd"
    
    local start_time=$(date +%s.%N)
    local start_mem=$(free -m | awk '/^Mem:/{print $3}')
    
    # Run the test and capture output
    if eval "$test_cmd" 2>&1 | tee "$RESULTS_DIR/${test_name}.log"; then
        local end_time=$(date +%s.%N)
        local end_mem=$(free -m | awk '/^Mem:/{print $3}')
        local duration=$(echo "$end_time - $start_time" | bc)
        local mem_delta=$((end_mem - start_mem))
        
        echo -e "${GREEN}✓ $test_name passed (${duration}s, ${mem_delta}MB memory delta)${NC}"
        echo "DURATION=$duration" >> "$RESULTS_DIR/${test_name}_metrics.txt"
        echo "MEMORY_DELTA=$mem_delta" >> "$RESULTS_DIR/${test_name}_metrics.txt"
        return 0
    else
        echo -e "${RED}✗ $test_name failed${NC}"
        return 1
    fi
}

# Build tests
echo -e "${BLUE}Building test binaries...${NC}"
nix develop --command cargo build --package cargo-vendormod --tests --no-default-features 2>&1 | tail -20

# Run unit tests
echo -e "\n${BLUE}=== Running Unit Tests ===${NC}"

# Test args module
run_test_workload "test_args" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib args -- --test-threads=1"

# Test repo_collection module  
run_test_workload "test_repo_collection" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib repo_collection -- --test-threads=1"

# Test git_wrapper module
run_test_workload "test_git_wrapper" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib git_wrapper -- --test-threads=1"

# Test rollup_lock module
run_test_workload "test_rollup_lock" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib rollup_lock -- --test-threads=1"

# Test workspace module
run_test_workload "test_workspace" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib workspace -- --test-threads=1"

# Test coverage module
run_test_workload "test_coverage" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib coverage -- --test-threads=1"

# Run all tests together
echo -e "\n${BLUE}=== Running All Unit Tests ===${NC}"
run_test_workload "test_all_units" "nix develop --command cargo test --package cargo-vendormod --no-default-features --lib -- --test-threads=1 2>&1"

# Generate report
echo -e "\n${BLUE}=== Generating Performance Report ===${NC}"
REPORT_FILE="$RESULTS_DIR/performance_report.md"

cat > "$REPORT_FILE" << EOF
# Cargo-Vendormod Workload Test Report
Generated: $(date)

## Test Results Summary

| Test | Status | Duration | Memory Delta |
|------|--------|----------|--------------|
EOF

for log in "$RESULTS_DIR"/*.log; do
    test_name=$(basename "$log" .log)
    if grep -q "test result: ok" "$log" 2>/dev/null; then
        status="PASSED"
    else
        status="FAILED"
    fi
    
    duration=$(grep "DURATION=" "$RESULTS_DIR/${test_name}_metrics.txt" 2>/dev/null | cut -d= -f2 || echo "N/A")
    mem_delta=$(grep "MEMORY_DELTA=" "$RESULTS_DIR/${test_name}_metrics.txt" 2>/dev/null | cut -d= -f2 || echo "N/A")
    
    echo "| $test_name | $status | ${duration}s | ${mem_delta}MB |" >> "$REPORT_FILE"
done

# Test count summary
total_tests=$(grep -r "#\[test\]" src/ | wc -l)
echo "" >> "$REPORT_FILE"
echo "## Coverage Summary" >> "$REPORT_FILE"
echo "- Total test functions: $total_tests" >> "$REPORT_FILE"
echo "- Modules with tests: $(grep -rl "#\[cfg(test)\]" src/ | wc -l)" >> "$REPORT_FILE"

echo "" >> "$REPORT_FILE"
echo "## Output Files" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"
ls -la "$RESULTS_DIR" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"

echo -e "\n${GREEN}======================================${NC}"
echo -e "${GREEN}  Report Generated: $REPORT_FILE${NC}"
echo -e "${GREEN}======================================${NC}"

cat "$REPORT_FILE"