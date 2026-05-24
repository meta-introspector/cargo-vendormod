use std::process::Command;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
struct ZkperfIntegrationResult {
    test_name: String,
    passed: bool,
    duration_ms: u64,
    output_files: Vec<String>,
    error_message: Option<String>,
    performance_metrics: Option<String>,
}

impl ZkperfIntegrationResult {
    fn new(test_name: String, passed: bool, duration_ms: u64, output_files: Vec<String>, error_message: Option<String>, performance_metrics: Option<String>) -> Self {
        ZkperfIntegrationResult {
            test_name,
            passed,
            duration_ms,
            output_files,
            error_message,
            performance_metrics,
        }
    }
}

fn main() {
    println!("🔗 Repository Mathematical Atlas - ZKperf Integration Test Suite");
    println!("==============================================================");
    
    let mut test_results = Vec::new();
    
    // Test 1: ZKperf recording integration
    println!("\n📹 Testing ZKperf Recording Integration...");
    let recording_result = test_zkperf_recording_integration();
    test_results.push(recording_result);
    
    // Test 2: ZKperf parsing integration
    println!("\n🔍 Testing ZKperf Parsing Integration...");
    let parsing_result = test_zkperf_parsing_integration();
    test_results.push(parsing_result);
    
    // Test 3: ZKperf coverage analysis
    println!("\n📊 Testing ZKperf Coverage Analysis Integration...");
    let coverage_result = test_zkperf_coverage_integration();
    test_results.push(coverage_result);
    
    // Test 4: ZKperf performance benchmarking
    println!("\n⚡ Testing ZKperf Performance Benchmarking Integration...");
    let benchmark_result = test_zkperf_benchmark_integration();
    test_results.push(benchmark_result);
    
    // Test 5: ZKperf trace analysis
    println!("\n🔬 Testing ZKperf Trace Analysis Integration...");
    let trace_result = test_zkperf_trace_integration();
    test_results.push(trace_result);
    
    // Generate comprehensive report
    generate_zkperf_integration_report(&test_results);
    
    // Print summary
    print_zkperf_integration_summary(&test_results);
}

fn test_zkperf_recording_integration() -> ZkperfIntegrationResult {
    let start = Instant::now();
    let test_name = "ZKperf Recording Integration";
    
    // Create output directory
    let output_dir = "./zkperf_integration_recording";
    if Path::new(output_dir).exists() {
        let _ = std::fs::remove_dir_all(output_dir);
    }
    
    // Run ZKperf recording on our repository mathematical atlas
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas"])
        .arg(output_dir)
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Check if expected output files were generated
                let expected_files = vec![
                    format!("{}/summary.cbor", output_dir),
                    format!("{}/func_0.cbor", output_dir),
                    format!("{}/instr_0.cbor", output_dir),
                ];
                
                let mut generated_files = Vec::new();
                for file_path in expected_files {
                    if Path::new(&file_path).exists() {
                        generated_files.push(file_path);
                    }
                }
                
                if generated_files.len() >= 2 {
                    // Calculate performance metrics
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let performance_metrics = format!("Recording completed in {}ms, {} files generated", duration_ms, generated_files.len());
                    
                    ZkperfIntegrationResult::new(
                        test_name.to_string(),
                        true,
                        duration_ms,
                        generated_files,
                        None,
                        Some(performance_metrics),
                    )
                } else {
                    ZkperfIntegrationResult::new(
                        test_name.to_string(),
                        false,
                        start.elapsed().as_millis() as u64,
                        generated_files,
                        Some("Not enough output files generated".to_string()),
                        None,
                    )
                }
            } else {
                ZkperfIntegrationResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    Vec::new(),
                    Some(format!("ZKperf recording failed: {}", String::from_utf8_lossy(&output.stderr))),
                    None,
                )
            }
        }
        Err(e) => {
            ZkperfIntegrationResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                Vec::new(),
                Some(format!("Failed to execute ZKperf recording: {}", e)),
                None,
            )
        }
    }
}

fn test_zkperf_parsing_integration() -> ZkperfIntegrationResult {
    let start = Instant::now();
    let test_name = "ZKperf Parsing Integration";
    
    // Create a test recording first
    let test_dir = "./zkperf_integration_parsing";
    if Path::new(test_dir).exists() {
        let _ = std::fs::remove_dir_all(test_dir);
    }
    
    // Generate test data by running a recording
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas"])
        .arg(test_dir)
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Now test parsing the generated data
                match Command::new("cargo")
                    .args(&["run", "--bin", "zkperf-parse", "--", "trace"])
                    .arg(format!("{}/summary.cbor", test_dir))
                    .current_dir("./zkperf")
                    .output() {
                    Ok(parse_output) => {
                        if parse_output.status.success() {
                            let parse_result = String::from_utf8_lossy(&parse_output.stdout);
                            
                            // Check if parsing contains expected data
                            let has_timestamps = parse_result.contains("ts");
                            let has_instructions = parse_result.contains("ip");
                            
                            if has_timestamps && has_instructions {
                                let duration_ms = start.elapsed().as_millis() as u64;
                                let performance_metrics = format!("Parsing completed in {}ms, trace data extracted", duration_ms);
                                
                                ZkperfIntegrationResult::new(
                                    test_name.to_string(),
                                    true,
                                    duration_ms,
                                    vec![format!("{}/summary.cbor", test_dir)],
                                    None,
                                    Some(performance_metrics),
                                )
                            } else {
                                ZkperfIntegrationResult::new(
                                    test_name.to_string(),
                                    false,
                                    start.elapsed().as_millis() as u64,
                                    vec![format!("{}/summary.cbor", test_dir)],
                                    Some("Parsing output missing expected data".to_string()),
                                    None,
                                )
                            }
                        } else {
                            ZkperfIntegrationResult::new(
                                test_name.to_string(),
                                false,
                                start.elapsed().as_millis() as u64,
                                vec![format!("{}/summary.cbor", test_dir)],
                                Some(format!("ZKperf parsing failed: {}", String::from_utf8_lossy(&parse_output.stderr))),
                                None,
                            )
                        }
                    }
                    Err(e) => {
                        ZkperfIntegrationResult::new(
                            test_name.to_string(),
                            false,
                            start.elapsed().as_millis() as u64,
                            vec![format!("{}/summary.cbor", test_dir)],
                            Some(format!("Failed to execute ZKperf parsing: {}", e)),
                            None,
                        )
                    }
                }
            } else {
                ZkperfIntegrationResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    Vec::new(),
                    Some(format!("Failed to generate test data: {}", String::from_utf8_lossy(&output.stderr))),
                    None,
                )
            }
        }
        Err(e) => {
            ZkperfIntegrationResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                Vec::new(),
                Some(format!("Failed to generate test data: {}", e)),
                None,
            )
        }
    }
}

fn test_zkperf_coverage_integration() -> ZkperfIntegrationResult {
    let start = Instant::now();
    let test_name = "ZKperf Coverage Analysis Integration";
    
    // Create output directory
    let output_dir = "./zkperf_integration_coverage";
    if Path::new(output_dir).exists() {
        let _ = std::fs::remove_dir_all(output_dir);
    }
    
    // Run coverage analysis using ZKperf
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-parse", "--", "trace"])
        .arg(format!("{}/summary.cbor", output_dir))
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                let coverage_result = String::from_utf8_lossy(&output.stdout);
                
                // Check if coverage analysis contains expected data
                let has_coverage_data = coverage_result.contains("ts") || coverage_result.contains("ip");
                
                if has_coverage_data {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let performance_metrics = format!("Coverage analysis completed in {}ms", duration_ms);
                    
                    ZkperfIntegrationResult::new(
                        test_name.to_string(),
                        true,
                        duration_ms,
                        vec![format!("{}/summary.cbor", output_dir)],
                        None,
                        Some(performance_metrics),
                    )
                } else {
                    ZkperfIntegrationResult::new(
                        test_name.to_string(),
                        false,
                        start.elapsed().as_millis() as u64,
                        vec![format!("{}/summary.cbor", output_dir)],
                        Some("Coverage analysis output missing expected data".to_string()),
                        None,
                    )
                }
            } else {
                ZkperfIntegrationResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    Vec::new(),
                    Some(format!("ZKperf coverage analysis failed: {}", String::from_utf8_lossy(&output.stderr))),
                    None,
                )
            }
        }
        Err(e) => {
            ZkperfIntegrationResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                Vec::new(),
                Some(format!("Failed to execute ZKperf coverage analysis: {}", e)),
                None,
            )
        }
    }
}

fn test_zkperf_benchmark_integration() -> ZkperfIntegrationResult {
    let start = Instant::now();
    let test_name = "ZKperf Performance Benchmarking Integration";
    
    // Create output directory
    let output_dir = "./zkperf_integration_benchmark";
    if Path::new(output_dir).exists() {
        let _ = std::fs::remove_dir_all(output_dir);
    }
    
    // Run performance benchmarking with multiple iterations
    let mut successful_iterations = 0;
    let mut total_duration = 0;
    
    for i in 0..3 {
        let iteration_dir = format!("{}/iteration_{}", output_dir, i);
        match Command::new("cargo")
            .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas"])
            .arg(&iteration_dir)
            .current_dir("./zkperf")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    successful_iterations += 1;
                    total_duration += output.status.code().unwrap_or(0) as u64;
                }
            }
            Err(_) => {
                // Continue with other iterations
            }
        }
    }
    
    if successful_iterations >= 2 {
        let duration_ms = start.elapsed().as_millis() as u64;
        let avg_duration = total_duration / successful_iterations;
        let performance_metrics = format!("Benchmarking completed in {}ms, {} successful iterations, avg {}ms/iteration", duration_ms, successful_iterations, avg_duration);
        
        ZkperfIntegrationResult::new(
            test_name.to_string(),
            true,
            duration_ms,
            (0..successful_iterations).map(|i| format!("{}/iteration_{}/summary.cbor", output_dir, i)).collect(),
            None,
            Some(performance_metrics),
        )
    } else {
        ZkperfIntegrationResult::new(
            test_name.to_string(),
            false,
            start.elapsed().as_millis() as u64,
            Vec::new(),
            Some(format!("Only {} successful benchmark iterations", successful_iterations)),
            None,
        )
    }
}

fn test_zkperf_trace_integration() -> ZkperfIntegrationResult {
    let start = Instant::now();
    let test_name = "ZKperf Trace Analysis Integration";
    
    // Create output directory
    let output_dir = "./zkperf_integration_trace";
    if Path::new(output_dir).exists() {
        let _ = std::fs::remove_dir_all(output_dir);
    }
    
    // Generate trace data
    match Command::new("cargo")
        .args(&["run", "--bin", "zkperf-record", "--", "run", "./simple_repository_mathematical_atlas"])
        .arg(output_dir)
        .current_dir("./zkperf")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Analyze the trace data
                match Command::new("cargo")
                    .args(&["run", "--bin", "zkperf-parse", "--", "trace"])
                    .arg(format!("{}/summary.cbor", output_dir))
                    .current_dir("./zkperf")
                    .output() {
                    Ok(parse_output) => {
                        if parse_output.status.success() {
                            let trace_analysis = String::from_utf8_lossy(&parse_output.stdout);
                            
                            // Check if trace analysis contains expected data
                            let has_timestamps = trace_analysis.contains("ts");
                            let has_instructions = trace_analysis.contains("ip");
                            
                            if has_timestamps && has_instructions {
                                let duration_ms = start.elapsed().as_millis() as u64;
                                let performance_metrics = format!("Trace analysis completed in {}ms, {} trace points extracted", duration_ms, trace_analysis.lines().count());
                                
                                ZkperfIntegrationResult::new(
                                    test_name.to_string(),
                                    true,
                                    duration_ms,
                                    vec![format!("{}/summary.cbor", output_dir)],
                                    None,
                                    Some(performance_metrics),
                                )
                            } else {
                                ZkperfIntegrationResult::new(
                                    test_name.to_string(),
                                    false,
                                    start.elapsed().as_millis() as u64,
                                    vec![format!("{}/summary.cbor", output_dir)],
                                    Some("Trace analysis output missing expected data".to_string()),
                                    None,
                                )
                            }
                        } else {
                            ZkperfIntegrationResult::new(
                                test_name.to_string(),
                                false,
                                start.elapsed().as_millis() as u64,
                                vec![format!("{}/summary.cbor", output_dir)],
                                Some(format!("Trace parsing failed: {}", String::from_utf8_lossy(&parse_output.stderr))),
                                None,
                            )
                        }
                    }
                    Err(e) => {
                        ZkperfIntegrationResult::new(
                            test_name.to_string(),
                            false,
                            start.elapsed().as_millis() as u64,
                            vec![format!("{}/summary.cbor", output_dir)],
                            Some(format!("Failed to execute trace analysis: {}", e)),
                            None,
                        )
                    }
                }
            } else {
                ZkperfIntegrationResult::new(
                    test_name.to_string(),
                    false,
                    start.elapsed().as_millis() as u64,
                    Vec::new(),
                    Some(format!("Failed to generate trace data: {}", String::from_utf8_lossy(&output.stderr))),
                    None,
                )
            }
        }
        Err(e) => {
            ZkperfIntegrationResult::new(
                test_name.to_string(),
                false,
                start.elapsed().as_millis() as u64,
                Vec::new(),
                Some(format!("Failed to generate trace data: {}", e)),
                None,
            )
        }
    }
}

fn generate_zkperf_integration_report(test_results: &[ZkperfIntegrationResult]) {
    let report_path = "./zkperf_integration_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas - ZKperf Integration Test Report\n\n");
    report.push_str("Generated on: ");
    report.push_str(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    report.push_str("\n\n## ZKperf Integration Test Summary\n\n");
    
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("- **Total ZKperf Integration Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Passed**: {}\n", passed_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    // Performance analysis
    let total_duration: u64 = test_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_tests as f64;
    
    report.push_str("## Performance Analysis\n\n");
    report.push_str(&format!("- **Total Test Time**: {}ms\n", total_duration));
    report.push_str(&format!("- **Average Test Time**: {:.1}ms\n\n", avg_duration));
    
    // File generation analysis
    let total_files: usize = test_results.iter().map(|r| r.output_files.len()).sum();
    report.push_str("## File Generation Analysis\n\n");
    report.push_str(&format!("- **Total Files Generated**: {}\n", total_files));
    
    // Count successful file generations
    let successful_files: usize = test_results.iter()
        .filter(|r| r.passed)
        .map(|r| r.output_files.len())
        .sum();
    report.push_str(&format!("- **Successful File Generations**: {}\n", successful_files));
    
    report.push_str("\n## Detailed ZKperf Integration Test Results\n\n");
    
    for result in test_results {
        let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", result.test_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str(&format!("- **Duration**: {}ms\n", result.duration_ms));
        report.push_str(&format!("- **Output Files**: {}\n", result.output_files.len()));
        
        for file in &result.output_files {
            report.push_str(&format!("  - {}\n", file));
        }
        
        if let Some(performance_metrics) = &result.performance_metrics {
            report.push_str(&format!("- **Performance Metrics**: {}\n", performance_metrics));
        }
        
        if let Some(error) = &result.error_message {
            report.push_str(&format!("- **Error**: {}\n", error));
        }
        
        report.push_str("\n");
    }
    
    // Integration recommendations
    report.push_str("## ZKperf Integration Recommendations\n\n");
    
    let passed_recording = test_results.iter().find(|r| r.test_name.contains("Recording")).map_or(false, |r| r.passed);
    let passed_parsing = test_results.iter().find(|r| r.test_name.contains("Parsing")).map_or(false, |r| r.passed);
    let passed_coverage = test_results.iter().find(|r| r.test_name.contains("Coverage")).map_or(false, |r| r.passed);
    let passed_benchmark = test_results.iter().find(|r| r.test_name.contains("Benchmark")).map_or(false, |r| r.passed);
    let passed_trace = test_results.iter().find(|r| r.test_name.contains("Trace")).map_or(false, |r| r.passed);
    
    if passed_recording {
        report.push_str("- ✅ **Recording Integration**: ZKperf recording is working correctly\n");
    } else {
        report.push_str("- ❌ **Recording Integration**: ZKperf recording needs improvement\n");
    }
    
    if passed_parsing {
        report.push_str("- ✅ **Parsing Integration**: ZKperf parsing is working correctly\n");
    } else {
        report.push_str("- ❌ **Parsing Integration**: ZKperf parsing needs improvement\n");
    }
    
    if passed_coverage {
        report.push_str("- ✅ **Coverage Integration**: ZKperf coverage analysis is working correctly\n");
    } else {
        report.push_str("- ❌ **Coverage Integration**: ZKperf coverage analysis needs improvement\n");
    }
    
    if passed_benchmark {
        report.push_str("- ✅ **Benchmark Integration**: ZKperf benchmarking is working correctly\n");
    } else {
        report.push_str("- ❌ **Benchmark Integration**: ZKperf benchmarking needs improvement\n");
    }
    
    if passed_trace {
        report.push_str("- ✅ **Trace Integration**: ZKperf trace analysis is working correctly\n");
    } else {
        report.push_str("- ❌ **Trace Integration**: ZKperf trace analysis needs improvement\n");
    }
    
    fs::write(report_path, report).expect("Failed to write ZKperf integration report");
    println!("📄 ZKperf integration report generated: {}", report_path);
}

fn print_zkperf_integration_summary(test_results: &[ZkperfIntegrationResult]) {
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    let total_duration: u64 = test_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_tests as f64;
    
    let total_files: usize = test_results.iter().map(|r| r.output_files.len()).sum();
    let successful_files: usize = test_results.iter()
        .filter(|r| r.passed)
        .map(|r| r.output_files.len())
        .sum();
    
    println!("\n🔗 ZKperf Integration Test Summary");
    println!("==================================");
    println!("📊 Total ZKperf Integration Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("📈 Success Rate: {:.1}%", success_rate);
    println!("⏱️  Total Time: {}ms", total_duration);
    println!("📊 Average Time: {:.1}ms", avg_duration);
    println!("📁 Total Files Generated: {}", total_files);
    println!("✅ Successful Files: {}", successful_files);
    
    if failed_tests == 0 {
        println!("🎉 All ZKperf integration tests passed! The Repository Mathematical Atlas has excellent ZKperf integration.");
    } else {
        println!("⚠️  {} ZKperf integration tests failed. Please review the ZKperf integration report for details.", failed_tests);
    }
    
    println!("\n📄 Detailed ZKperf integration report available in: ./zkperf_integration_report.md");
}