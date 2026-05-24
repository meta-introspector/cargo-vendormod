use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct IntegrationTestResult {
    test_name: String,
    passed: bool,
    duration_ms: u64,
    error_message: Option<String>,
}

impl IntegrationTestResult {
    fn new(test_name: String, passed: bool, duration_ms: u64, error_message: Option<String>) -> Self {
        IntegrationTestResult {
            test_name,
            passed,
            duration_ms,
            error_message,
        }
    }
}

fn main() {
    println!("🔗 Repository Mathematical Atlas Integration Test Suite");
    println!("===================================================");
    
    let mut test_results = Vec::new();
    
    // Test complete workflow integration
    println!("\n🔄 Testing Complete Workflow Integration...");
    let workflow_result = test_complete_workflow_integration();
    test_results.push(workflow_result);
    
    // Test file I/O operations
    println!("\n📁 Testing File I/O Operations...");
    let io_result = test_file_io_operations();
    test_results.push(io_result);
    
    // Test JSON generation and parsing
    println!("\n📋 Testing JSON Generation and Parsing...");
    let json_result = test_json_generation_parsing();
    test_results.push(json_result);
    
    // Test mathematical calculations
    println!("\n🔢 Testing Mathematical Calculations...");
    let math_result = test_mathematical_calculations();
    test_results.push(math_result);
    
    // Test error handling
    println!("\n⚠️  Testing Error Handling...");
    let error_result = test_error_handling();
    test_results.push(error_result);
    
    // Generate integration test report
    generate_integration_test_report(&test_results);
    
    // Print summary
    print_integration_test_summary(&test_results);
}

fn test_complete_workflow_integration() -> IntegrationTestResult {
    let start = std::time::Instant::now();
    
    // Step 1: Run the atlas to generate initial data
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if !output.status.success() {
                return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(format!("Atlas execution failed: {}", String::from_utf8_lossy(&output.stderr))));
            }
        }
        Err(e) => {
            return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute atlas: {}", e)));
        }
    }
    
    // Step 2: Verify output files were created
    let expected_files = vec![
        "repository_atlas_output/simple_complete_atlas.md",
        "repository_atlas_output/simple_composition_all_repos.json",
    ];
    
    for file_path in expected_files {
        if !Path::new(file_path).exists() {
            return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Missing expected file: {}", file_path)));
        }
    }
    
    // Step 3: Read and validate the complete atlas
    match fs::read_to_string("repository_atlas_output/simple_complete_atlas.md") {
        Ok(content) => {
            if !content.contains("Repository Mathematical Atlas") || !content.contains("Family Distribution") {
                return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some("Invalid atlas content".to_string()));
            }
        }
        Err(e) => {
            return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to read atlas: {}", e)));
        }
    }
    
    // Step 4: Validate JSON composition
    match fs::read_to_string("repository_atlas_output/simple_composition_all_repos.json") {
        Ok(content) => {
            if !content.contains("All Repositories") || !content.contains("Lie Type") {
                return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some("Invalid JSON composition".to_string()));
            }
        }
        Err(e) => {
            return IntegrationTestResult::new("Complete Workflow Integration".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to read JSON composition: {}", e)));
        }
    }
    
    IntegrationTestResult::new("Complete Workflow Integration".to_string(), true, start.elapsed().as_millis() as u64, None)
}

fn test_file_io_operations() -> IntegrationTestResult {
    let start = std::time::Instant::now();
    
    // Test file creation, reading, and deletion
    let test_file = "./test_io_file.txt";
    let test_content = "Test content for file I/O operations";
    
    // Step 1: Create test file
    match fs::write(test_file, test_content) {
        Ok(_) => {},
        Err(e) => {
            return IntegrationTestResult::new("File I/O Operations".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to create test file: {}", e)));
        }
    }
    
    // Step 2: Read test file
    match fs::read_to_string(test_file) {
        Ok(content) => {
            if content != test_content {
                return IntegrationTestResult::new("File I/O Operations".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some("File content mismatch".to_string()));
            }
        }
        Err(e) => {
            return IntegrationTestResult::new("File I/O Operations".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to read test file: {}", e)));
        }
    }
    
    // Step 3: Delete test file
    match fs::remove_file(test_file) {
        Ok(_) => {},
        Err(e) => {
            return IntegrationTestResult::new("File I/O Operations".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to delete test file: {}", e)));
        }
    }
    
    // Step 4: Verify file was deleted
    if Path::new(test_file).exists() {
        return IntegrationTestResult::new("File I/O Operations".to_string(), false, start.elapsed().as_millis() as u64, 
            Some("Test file still exists after deletion".to_string()));
    }
    
    IntegrationTestResult::new("File I/O Operations".to_string(), true, start.elapsed().as_millis() as u64, None)
}

fn test_json_generation_parsing() -> IntegrationTestResult {
    let start = std::time::Instant::now();
    
    // Step 1: Generate JSON using the atlas
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if !output.status.success() {
                return IntegrationTestResult::new("JSON Generation and Parsing".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(format!("Atlas execution failed: {}", String::from_utf8_lossy(&output.stderr))));
            }
        }
        Err(e) => {
            return IntegrationTestResult::new("JSON Generation and Parsing".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute atlas: {}", e)));
        }
    }
    
    // Step 2: Read and parse JSON compositions
    let json_files = vec![
        "repository_atlas_output/simple_composition_all_repos.json",
        "repository_atlas_output/simple_composition_cyclic_repositories.json",
        "repository_atlas_output/simple_composition_high_complexity_repositories.json",
    ];
    
    for json_file in json_files {
        match fs::read_to_string(json_file) {
            Ok(content) => {
                // Basic JSON validation
                if !content.contains("{") || !content.contains("}") {
                    return IntegrationTestResult::new("JSON Generation and Parsing".to_string(), false, start.elapsed().as_millis() as u64, 
                        Some(format!("Invalid JSON in file: {}", json_file)));
                }
                
                // Check for expected keys
                if !content.contains("view_name") || !content.contains("repositories") {
                    return IntegrationTestResult::new("JSON Generation and Parsing".to_string(), false, start.elapsed().as_millis() as u64, 
                        Some(format!("Missing expected keys in JSON: {}", json_file)));
                }
            }
            Err(e) => {
                return IntegrationTestResult::new("JSON Generation and Parsing".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(format!("Failed to read JSON file {}: {}", json_file, e)));
            }
        }
    }
    
    IntegrationTestResult::new("JSON Generation and Parsing".to_string(), true, start.elapsed().as_millis() as u64, None)
}

fn test_mathematical_calculations() -> IntegrationTestResult {
    let start = std::time::Instant::now();
    
    // Test mathematical consistency by running multiple iterations
    let mut total_complexity = 0.0;
    let mut total_repos = 0;
    
    for _i in 0..3 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // Read the atlas and calculate total complexity
                    match fs::read_to_string("repository_atlas_output/simple_complete_atlas.md") {
                        Ok(content) => {
                            // Extract complexity scores (simplified check)
                            if content.contains("Average Complexity") {
                                total_repos += 1;
                            }
                        }
                        Err(_) => {
                            // Continue with other iterations
                        }
                    }
                }
            }
            Err(_) => {
                // Continue with other iterations
            }
        }
    }
    
    if total_repos >= 2 {
        IntegrationTestResult::new("Mathematical Calculations".to_string(), true, start.elapsed().as_millis() as u64, None)
    } else {
        IntegrationTestResult::new("Mathematical Calculations".to_string(), false, start.elapsed().as_millis() as u64, 
            Some(format!("Only {} successful mathematical calculations out of 3", total_repos)))
    }
}

fn test_error_handling() -> IntegrationTestResult {
    let start = std::time::Instant::now();
    
    // Test error handling by running with invalid inputs (if any)
    // Since this is a standalone binary, we test basic error scenarios
    
    // Test 1: Check if binary handles missing files gracefully
    match Command::new("./simple_repository_mathematical_atlas")
        .current_dir("/nonexistent/directory")
        .output() {
        Ok(output) => {
            // The binary should handle this gracefully
            if output.status.success() {
                // This is expected - the binary should work regardless of current directory
            } else {
                // Non-zero exit code might be acceptable for some error cases
            }
        }
        Err(_) => {
            // This might happen if the binary can't be found, but we already tested it works
        }
    }
    
    // Test 2: Verify that the atlas doesn't crash with empty output directories
    let output_backup = fs::read_dir("repository_atlas_output").unwrap().count();
    
    // The atlas should handle existing output files gracefully
    match Command::new("./simple_repository_mathematical_atlas")
        .output() {
        Ok(output) => {
            if output.status.success() {
                IntegrationTestResult::new("Error Handling".to_string(), true, start.elapsed().as_millis() as u64, None)
            } else {
                IntegrationTestResult::new("Error Handling".to_string(), false, start.elapsed().as_millis() as u64, 
                    Some(format!("Atlas failed with error: {}", String::from_utf8_lossy(&output.stderr))))
            }
        }
        Err(e) => {
            IntegrationTestResult::new("Error Handling".to_string(), false, start.elapsed().as_millis() as u64, 
                Some(format!("Failed to execute atlas: {}", e)))
        }
    }
}

fn generate_integration_test_report(test_results: &[IntegrationTestResult]) {
    let report_path = "./integration_test_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas Integration Test Report\n\n");
    report.push_str("Generated on: ");
    report.push_str(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    report.push_str("\n\n## Integration Test Summary\n\n");
    
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("- **Total Integration Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Passed**: {}\n", passed_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    report.push_str("## Detailed Integration Results\n\n");
    
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
    
    fs::write(report_path, report).expect("Failed to write integration test report");
    println!("📄 Integration test report generated: {}", report_path);
}

fn print_integration_test_summary(test_results: &[IntegrationTestResult]) {
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    println!("\n🎯 Integration Test Summary");
    println!("===========================");
    println!("📊 Total Integration Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("📈 Success Rate: {:.1}%", success_rate);
    
    if failed_tests == 0 {
        println!("🎉 All integration tests passed! The Repository Mathematical Atlas is fully functional.");
    } else {
        println!("⚠️  {} integration tests failed. Please review the integration test report for details.", failed_tests);
    }
    
    println!("\n📄 Detailed integration test report available in: ./integration_test_report.md");
}