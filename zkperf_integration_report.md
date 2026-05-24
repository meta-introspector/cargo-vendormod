# Repository Mathematical Atlas - ZKperf Integration Test Report

Generated on: 1779359154

## ZKperf Integration Test Summary

- **Total ZKperf Integration Tests**: 5
- **Passed**: 1
- **Failed**: 4
- **Success Rate**: 20.0%

## Performance Analysis

- **Total Test Time**: 13372ms
- **Average Test Time**: 2674.4ms

## File Generation Analysis

- **Total Files Generated**: 6
- **Successful File Generations**: 3

## Detailed ZKperf Integration Test Results

### ZKperf Recording Integration
- **Status**: ❌ FAIL
- **Duration**: 2137ms
- **Output Files**: 0
- **Error**: Not enough output files generated

### ZKperf Parsing Integration
- **Status**: ❌ FAIL
- **Duration**: 2253ms
- **Output Files**: 1
  - ./zkperf_integration_parsing/summary.cbor
- **Error**: Parsing output missing expected data

### ZKperf Coverage Analysis Integration
- **Status**: ❌ FAIL
- **Duration**: 129ms
- **Output Files**: 1
  - ./zkperf_integration_coverage/summary.cbor
- **Error**: Coverage analysis output missing expected data

### ZKperf Performance Benchmarking Integration
- **Status**: ✅ PASS
- **Duration**: 6596ms
- **Output Files**: 3
  - ./zkperf_integration_benchmark/iteration_0/summary.cbor
  - ./zkperf_integration_benchmark/iteration_1/summary.cbor
  - ./zkperf_integration_benchmark/iteration_2/summary.cbor
- **Performance Metrics**: Benchmarking completed in 6596ms, 3 successful iterations, avg 0ms/iteration

### ZKperf Trace Analysis Integration
- **Status**: ❌ FAIL
- **Duration**: 2257ms
- **Output Files**: 1
  - ./zkperf_integration_trace/summary.cbor
- **Error**: Trace analysis output missing expected data

## ZKperf Integration Recommendations

- ❌ **Recording Integration**: ZKperf recording needs improvement
- ❌ **Parsing Integration**: ZKperf parsing needs improvement
- ❌ **Coverage Integration**: ZKperf coverage analysis needs improvement
- ✅ **Benchmark Integration**: ZKperf benchmarking is working correctly
- ❌ **Trace Integration**: ZKperf trace analysis needs improvement
