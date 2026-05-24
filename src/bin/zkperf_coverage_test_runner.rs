use std::process::Command;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
struct ZkperfTestResult {
    test_name: String,
    passed: bool,
    duration_ms: u64,
    coverage_data: Option<String>,
    perf_data: Option<String>,
    error_message: Option<String>,
}

impl ZkperfTestResult {
    fn new(test_name: String, passed: bool, duration_ms: u64, coverage_data: Option<String>, perf_data: Option<String>, error_message: Option<String>) -> Self {
        ZkperfTestResult {
            test_name,
            passed,
            duration_ms,
            coverage_data,
            perf_data,
            error_message,
        }
    }
}

fn main() {
    println!("🔍 Repository Mathematical Atlas - ZKperf Coverage & Performance Testing");
    println!("=====================================================================");
    
    let mut test_results = Vec::new();
    
    // Test 1: Basic coverage recording
    println!("\n📊 Testing Basic Coverage Recording...");
    let coverage_result = test_basic_coverage_recording();
    test_results.push(coverage_result);
    
    // Test 2: Performance recording with ZKperf
    println!("\n⚡ Testing Performance Recording...");
    let perf_result = test_performance_recording();
    test_results.push(perf_result);
    
    // Test 3: Coverage analysis
    println!("\n🔬 Testing Coverage Analysis...");
    let analysis_result = test_coverage_analysis();
    test_results.push(analysis_result);
    
    // Test 4: ZKperf integration
    println!("\n🔗 Testing ZKperf Integration...");
    let integration_result = test_zkperf_integration();
    test_results.push(integration_result);
    
    // Test 5: Performance benchmarking with ZKperf
    println!("\n🎯 Testing Performance Benchmarking...");
    let benchmark_result = test_performance_benchmarking();
    test_results.push(benchmark_result);
    
    // Generate comprehensive report
    generate_zkperf_test_report(&test_results);
    
    // Print summary
    print_zkperf_test_summary(&test_results);
}

fn test_basic_coverage_recording() -> ZkperfTestResult {
    let start = Instant::now();
    let test_name = "Basic Coverage Recording";
    
    // Create output directory for coverage data
    let coverage_dir = "./zkperf_coverage_output";
    if Path::new(coverage_dir).exists() {
        let _ = std::fs::remove_dir_all(coverage_dir);
    }
    
    // Run the repository mathematical atlas with ZKperf recording
    match Command::new("cargo")
        .args(&["run", "--bin", "simple_repository_mathematical_atlas"])
        .current_dir(".")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Check if coverage data was generated
                let coverage_files = vec![
                    format!("{}/summary.cbor", coverage_dir),
                    format!("{}/func_0.cbor", coverage_dir),
                    format!("{}/instr_0.cbor", coverage_dir),
                ];
                
                let mut coverage_generated = false;
                for file_path in coverage_files {
                    if Path::new(&file_path).exists() {
                        coverage_generated = true;
                        break;
                    }
                }
                
                if coverage_generated {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        true,
                        start.elapsed().as_millis() as u64,
                        Some("Coverage data generated successfully".to_string()),
                        None,
                        None,
                    )
                } else {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        false,
                        start.elapsed().as_millis() as u64,
                        None,
                        None,
                        Some("No coverage data files found".to_string()),
                    )
                }
            } else {
                ZkperfTestResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    None,
                    None,
                    Some(format!("Atlas execution failed: {}", String::from_utf8_lossy(&output.stderr))),
                )
            }
        }
        Err(e) => {
            ZkperfTestResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                None,
                None,
                Some(format!("Failed to execute atlas: {}", e)),
            )
        }
    }
}

fn test_performance_recording() -> ZkperfTestResult {
    let start = Instant::now();
    let test_name = "Performance Recording";
    
    // Create output directory for performance data
    let perf_dir = "./zkperf_perf_output";
    if Path::new(perf_dir).exists() {
        let _ = std::fs::remove_dir_all(perf_dir);
    }
    
    // Use ZKperf to record performance data
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas", perf_dir])
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Check if performance data was generated
                let perf_files = vec![
                    format!("{}/summary.cbor", perf_dir),
                    format!("{}/trace.cbor", perf_dir),
                ];
                
                let mut perf_generated = false;
                for file_path in perf_files {
                    if Path::new(&file_path).exists() {
                        perf_generated = true;
                        break;
                    }
                }
                
                if perf_generated {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        true,
                        start.elapsed().as_millis() as u64,
                        None,
                        Some("Performance data generated successfully".to_string()),
                        None,
                    )
                } else {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        false,
                        start.elapsed().as_millis() as u64,
                        None,
                        None,
                        Some("No performance data files found".to_string()),
                    )
                }
            } else {
                ZkperfTestResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    None,
                    None,
                    Some(format!("ZKperf recording failed: {}", String::from_utf8_lossy(&output.stderr))),
                )
            }
        }
        Err(e) => {
            ZkperfTestResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                None,
                None,
                Some(format!("Failed to execute ZKperf: {}", e)),
            )
        }
    }
}

fn test_coverage_analysis() -> ZkperfTestResult {
    let start = Instant::now();
    let test_name = "Coverage Analysis";
    
    // Create output directory for analysis
    let analysis_dir = "./zkperf_analysis_output";
    if Path::new(analysis_dir).exists() {
        let _ = std::fs::remove_dir_all(analysis_dir);
    }
    
    // Run coverage analysis using ZKperf tools
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-parse", "--", "trace", "./zkperf_perf_output/perf.data"])
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Check if analysis output contains expected data
                let analysis_output = String::from_utf8_lossy(&output.stdout);
                let has_timestamps = analysis_output.contains("ts");
                let has_instructions = analysis_output.contains("ip");
                
                if has_timestamps && has_instructions {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        true,
                        start.elapsed().as_millis() as u64,
                        Some("Coverage analysis completed successfully".to_string()),
                        None,
                        None,
                    )
                } else {
                    ZkperfTestResult::new(
                        test_name.to_string(),
                        false,
                        start.elapsed().as_millis() as u64,
                        None,
                        None,
                        Some("Analysis output missing expected data".to_string()),
                    )
                }
            } else {
                ZkperfTestResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    None,
                    None,
                    Some(format!("Coverage analysis failed: {}", String::from_utf8_lossy(&output.stderr))),
                )
            }
        }
        Err(e) => {
            ZkperfTestResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                None,
                None,
                Some(format!("Failed to execute coverage analysis: {}", e)),
            )
        }
    }
}

fn test_zkperf_integration() -> ZkperfTestResult {
    let start = Instant::now();
    let test_name = "ZKperf Integration";
    
    // Test ZKperf integration by running multiple recording scenarios
    let scenarios = vec![
        ("basic", vec!["./simple_repository_mathematical_atlas"]),
        ("with_output", vec!["./simple_repository_mathematical_atlas", ">", "/dev/null"]),
    ];
    
    let mut successful_scenarios = 0;
    
    for (scenario_name, command) in scenarios {
        let scenario_dir = format!("./zkperf_integration_{}", scenario_name);
        if Path::new(&scenario_dir).exists() {
            let _ = std::fs::remove_dir_all(&scenario_dir);
        }
        
        match Command::new("cargo")
            .args(&["run", "--bin", "zkperf-record", "--", "run"])
            .args(command)
            .arg(&scenario_dir)
            .current_dir("./zkperf")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    successful_scenarios += 1;
                }
            }
            Err(_) => {
                // Continue with other scenarios
            }
        }
    }
    
    if successful_scenarios >= 1 {
        ZkperfTestResult::new(
            test_name.to_string(),
            true,
            start.elapsed().as_millis() as u64,
            Some(format!("{} scenarios successful", successful_scenarios)),
            None,
            None,
        )
    } else {
        ZkperfTestResult::new(
            test_name.to_string(),
            false,
            start.elapsed().as_millis() as u64,
            None,
            None,
            Some("No integration scenarios successful".to_string()),
        )
    }
}

fn test_performance_benchmarking() -> ZkperfTestResult {
    let start = Instant::now();
    let test_name = "Performance Benchmarking";
    
    // Create output directory for benchmarking
    let benchmark_dir = "./zkperf_benchmark_output";
    if Path::new(benchmark_dir).exists() {
        let _ = std::fs::remove_dir_all(benchmark_dir);
    }
    
    // Run performance benchmarking with multiple iterations
    let mut successful_benchmarks = 0;
    
    for i in 0..3 {
        let iteration_dir = format!("{}/iteration_{}", benchmark_dir, i);
        match Command::new("cargo")
            .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas", &iteration_dir])
            .current_dir("./zkperf")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // Check if benchmark data was generated
                    if Path::new(&format!("{}/summary.cbor", iteration_dir)).exists() {
                        successful_benchmarks += 1;
                    }
                }
            }
            Err(_) => {
                // Continue with other iterations
            }
        }
    }
    
    if successful_benchmarks >= 2 {
        ZkperfTestResult::new(
            test_name.to_string(),
            true,
            start.elapsed().as_millis() as u64,
            Some(format!("{} benchmark iterations successful", successful_benchmarks)),
            None,
            None,
        )
    } else {
        ZkperfTestResult::new(
            test_name.to_string(),
            false,
            start.elapsed().as_millis() as u64,
            None,
            None,
            Some(format!("Only {} successful benchmark iterations", successful_benchmarks)),
        )
    }
}

fn generate_zkperf_test_report(test_results: &[ZkperfTestResult]) {
    let report_path = "./zkperf_test_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas - ZKperf Coverage & Performance Test Report\n\n");
    report.push_str("Generated on: ");
    report.push_str(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    report.push_str("\n\n## ZKperf Test Summary\n\n");
    
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("- **Total ZKperf Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Passed**: {}\n", passed_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    // Performance analysis
    let total_duration: u64 = test_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_tests as f64;
    
    report.push_str("## Performance Analysis\n\n");
    report.push_str(&format!("- **Total Test Time**: {}ms\n", total_duration));
    report.push_str(&format!("- **Average Test Time**: {:.1}ms\n\n", avg_duration));
    
    report.push_str("## Detailed ZKperf Test Results\n\n");
    
    for result in test_results {
        let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", result.test_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str(&format!("- **Duration**: {}ms\n", result.duration_ms));
        
        if let Some(coverage_data) = &result.coverage_data {
            report.push_str(&format!("- **Coverage Data**: {}\n", coverage_data));
        }
        
        if let Some(perf_data) = &result.perf_data {
            report.push_str(&format!("- **Performance Data**: {}\n", perf_data));
        }
        
        if let Some(error) = &result.error_message {
            report.push_str(&format!("- **Error**: {}\n", error));
        }
        
        report.push_str("\n");
    }
    
    // ZKperf integration recommendations
    report.push_str("## ZKperf Integration Recommendations\n\n");
    
    let passed_coverage = test_results.iter().filter(|r| r.coverage_data.is_some()).count();
    let passed_performance = test_results.iter().filter(|r| r.perf_data.is_some()).count();
    
    if passed_coverage >= 3 {
        report.push_str("- ✅ **Excellent Coverage Integration**: ZKperf coverage recording is working well\n");
    } else if passed_coverage >= 1 {
        report.push_str("- ⚠️ **Good Coverage Integration**: Basic coverage recording works\n");
    } else {
        report.push_str("- ❌ **Poor Coverage Integration**: Coverage recording needs improvement\n");
    }
    
    if passed_performance >= 3 {
        report.push_str("- ✅ **Excellent Performance Integration**: ZKperf performance recording is working well\n");
    } else if passed_performance >= 1 {
        report.push_str("- ⚠️ **Good Performance Integration**: Basic performance recording works\n");
    } else {
        report.push_str("- ❌ **Poor Performance Integration**: Performance recording needs improvement\n");
    }
    
    fs::write(report_path, report).expect("Failed to write ZKperf test report");
    println!("📄 ZKperf test report generated: {}", report_path);
}

fn print_zkperf_test_summary(test_results: &[ZkperfTestResult]) {
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    let total_duration: u64 = test_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_tests as f64;
    
    println!("\n🔍 ZKperf Test Summary");
    println!("=====================");
    println!("📊 Total ZKperf Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("📈 Success Rate: {:.1}%", success_rate);
    println!("⏱️  Total Time: {}ms", total_duration);
    println!("📊 Average Time: {:.1}ms", avg_duration);
    
    if failed_tests == 0 {
        println!("🎉 All ZKperf tests passed! The Repository Mathematical Atlas has excellent ZKperf integration.");
    } else {
        println!("⚠️  {} ZKperf tests failed. Please review the ZKperf test report for details.", failed_tests);
    }
    
    println!("\n📄 Detailed ZKperf test report available in: ./zkperf_test_report.md");
}