use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct TestResult {
    test_name: String,
    passed: bool,
    duration_ms: u64,
    error_message: Option<String>,
}

impl TestResult {
    fn new(test_name: String, passed: bool, duration_ms: u64, error_message: Option<String>) -> Self {
        TestResult {
            test_name,
            passed,
            duration_ms,
            error_message,
        }
    }
}

fn main() {
    println!("🧪 Repository Mathematical Atlas Test Suite");
    println!("==========================================");
    
    let mut test_results = Vec::new();
    
    // Test standalone binary compilation
    println!("\n📋 Testing Standalone Binary Compilation...");
    let compile_result = test_standalone_compilation();
    test_results.push(compile_result);
    
    // Test basic functionality
    println!("\n🔍 Testing Basic Functionality...");
    let basic_result = test_basic_functionality();
    test_results.push(basic_result);
    
    // Test output generation
    println!("\n📤 Testing Output Generation...");
    let output_result = test_output_generation();
    test_results.push(output_result);
    
    // Test mathematical consistency
    println!("\n🔢 Testing Mathematical Consistency...");
    let math_result = test_mathematical_consistency();
    test_results.push(math_result);
    
    // Test performance
    println!("\n⚡ Testing Performance...");
    let perf_result = test_performance();
    test_results.push(perf_result);
    
    // Generate test report
    generate_test_report(&test_results);
    
    // Print summary
    print_test_summary(&test_results);
}

fn test_standalone_compilation() -> TestResult {
    let start = std::time::Instant::now();
    
    match Command::new("nix")
        .args(&["develop", "--command", "rustc", "-o", "test_atlas", "src/bin/simple_repository_mathematical_atlas.rs"])
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Clean up test binary
                let _ = std::fs::remove_file("test_atlas");
                TestResult::new("Standalone Compilation".to_string(), true, start.elapsed().as_millis() as u64, None)
            } else {
                TestResult::new("Standalone Compilation".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }
        Err(e) => {
            TestResult::new("Standalone Compilation".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute compiler: {}", e)))
        }
    }
}

fn test_basic_functionality() -> TestResult {
    let start = std::time::Instant::now();
    
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                // Check for expected output
                let has_atlas_header = stdout.contains("Repository Mathematical Atlas");
                let has_views_list = stdout.contains("Available Views:");
                let has_completion = stdout.contains("Analysis Complete!");
                
                if has_atlas_header && has_views_list && has_completion {
                    TestResult::new("Basic Functionality".to_string(), true, start.elapsed().as_millis() as u64, None)
                } else {
                    TestResult::new("Basic Functionality".to_string(), false, start.elapsed().as_millis() as u64, 
                        Some("Missing expected output elements".to_string()))
                }
            } else {
                TestResult::new("Basic Functionality".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }
        Err(e) => {
            TestResult::new("Basic Functionality".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute binary: {}", e)))
        }
    }
}

fn test_output_generation() -> TestResult {
    let start = std::time::Instant::now();
    
    // Run the atlas to generate outputs
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Check if output files were created
                let expected_files = vec![
                    "repository_atlas_output/simple_complete_atlas.md",
                    "repository_atlas_output/simple_composition_all_repos.json",
                    "repository_atlas_output/simple_composition_cyclic_repositories.json",
                    "repository_atlas_output/simple_composition_high_complexity_repositories.json",
                    "repository_atlas_output/simple_composition_javascript_repositories.json",
                ];
                
                let mut all_files_exist = true;
                for file_path in expected_files {
                    if !Path::new(file_path).exists() {
                        all_files_exist = false;
                        break;
                    }
                }
                
                if all_files_exist {
                    TestResult::new("Output Generation".to_string(), true, start.elapsed().as_millis() as u64, None)
                } else {
                    TestResult::new("Output Generation".to_string(), false, start.elapsed().as_millis() as u64, 
                        Some("Some output files are missing".to_string()))
                }
            } else {
                TestResult::new("Output Generation".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }
        Err(e) => {
            TestResult::new("Output Generation".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute binary: {}", e)))
        }
    }
}

fn test_mathematical_consistency() -> TestResult {
    let start = std::time::Instant::now();
    
    // Run the atlas
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if output.status.success() {
                // Read the generated atlas data
                match fs::read_to_string("repository_atlas_output/simple_complete_atlas.md") {
                    Ok(atlas_content) => {
                        // Check for mathematical consistency
                        let has_family_distribution = atlas_content.contains("Family Distribution");
                        let has_complexity_stats = atlas_content.contains("Complexity Statistics");
                        let has_repository_details = atlas_content.contains("Repository Details");
                        let has_group_theory = atlas_content.contains("Group Theory Description");
                        
                        if has_family_distribution && has_complexity_stats && has_repository_details && has_group_theory {
                            TestResult::new("Mathematical Consistency".to_string(), true, start.elapsed().as_millis() as u64, None)
                        } else {
                            TestResult::new("Mathematical Consistency".to_string(), false, start.elapsed().as_millis() as u64, 
                                Some("Missing mathematical elements in atlas".to_string()))
                        }
                    }
                    Err(e) => {
                        TestResult::new("Mathematical Consistency".to_string(), false, start.elapsed().as_millis() as u64, 
                            Some(format!("Failed to read atlas data: {}", e)))
                    }
                }
            } else {
                TestResult::new("Mathematical Consistency".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }
        Err(e) => {
            TestResult::new("Mathematical Consistency".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute binary: {}", e)))
        }
    }
}

fn test_performance() -> TestResult {
    let start = std::time::Instant::now();
    
    // Run the atlas multiple times to test performance
    let mut total_duration = std::time::Duration::new(0, 0);
    let mut successful_runs = 0;
    
    for _i in 0..5 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    successful_runs += 1;
                    total_duration += start.elapsed();
                }
            }
            Err(_) => {
                // Continue with other runs
            }
        }
    }
    
    if successful_runs >= 3 { // At least 3 successful runs
        let avg_duration = total_duration / successful_runs as u32;
        TestResult::new("Performance".to_string(), true, avg_duration.as_millis() as u64, None)
    } else {
        TestResult::new("Performance".to_string(), false, start.elapsed().as_millis() as u64, 
            Some(format!("Only {} successful runs out of 5", successful_runs)))
    }
}

fn generate_test_report(test_results: &[TestResult]) {
    let report_path = "./test_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas Test Report\n\n");
    report.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().to_rfc3339()));
    report.push_str("\n\n## Test Summary\n\n");
    
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("- **Total Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Passed**: {}\n", passed_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    report.push_str("## Detailed Results\n\n");
    
    for result in test_results {
        let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", result.test_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str(&format!("- **Duration**: {}ms\n", result.duration_ms));
        
        if let Some(error) = &result.error_message {
            report.push_str(&format!("- **Error**: {}\n", error));
        }
        
        report.push_str("\n");
    }
    
    fs::write(report_path, report).expect("Failed to write test report");
    println!("📄 Test report generated: {}", report_path);
}

fn print_test_summary(test_results: &[TestResult]) {
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    println!("\n🎯 Test Summary");
    println!("===============");
    println!("📊 Total Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("📈 Success Rate: {:.1}%", success_rate);
    
    if failed_tests == 0 {
        println!("🎉 All tests passed! The Repository Mathematical Atlas is working correctly.");
    } else {
        println!("⚠️  {} tests failed. Please review the test report for details.", failed_tests);
    }
    
    println!("\n📄 Detailed test report available in: ./test_report.md");
}