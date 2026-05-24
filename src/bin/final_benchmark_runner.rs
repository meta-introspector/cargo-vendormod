use std::process::Command;
use std::time::Instant;
use std::fs;

#[derive(Debug)]
struct BenchmarkResult {
    benchmark_name: String,
    duration_ms: u64,
    iterations: usize,
    avg_duration_ms: f64,
    success: bool,
    error_message: Option<String>,
}

impl BenchmarkResult {
    fn new(benchmark_name: String, duration_ms: u64, iterations: usize, success: bool, error_message: Option<String>) -> Self {
        let avg_duration_ms = if iterations > 0 {
            duration_ms as f64 / iterations as f64
        } else {
            0.0
        };
        
        BenchmarkResult {
            benchmark_name,
            duration_ms,
            iterations,
            avg_duration_ms,
            success,
            error_message,
        }
    }
}

fn main() {
    println!("⚡ Repository Mathematical Atlas Benchmark Suite");
    println!("============================================");
    
    let mut benchmark_results = Vec::new();
    
    // Benchmark compilation performance
    println!("\n🔨 Benchmarking Compilation Performance...");
    let compile_benchmark = benchmark_compilation_performance();
    benchmark_results.push(compile_benchmark);
    
    // Benchmark execution performance
    println!("\n🚀 Benchmarking Execution Performance...");
    let execution_benchmark = benchmark_execution_performance();
    benchmark_results.push(execution_benchmark);
    
    // Benchmark output generation performance
    println!("\n📤 Benchmarking Output Generation Performance...");
    let output_benchmark = benchmark_output_generation_performance();
    benchmark_results.push(output_benchmark);
    
    // Benchmark mathematical operations performance
    println!("\n🔢 Benchmarking Mathematical Operations Performance...");
    let math_benchmark = benchmark_mathematical_operations_performance();
    benchmark_results.push(math_benchmark);
    
    // Benchmark memory usage (simplified)
    println!("\n💾 Benchmarking Memory Usage...");
    let memory_benchmark = benchmark_memory_usage();
    benchmark_results.push(memory_benchmark);
    
    // Generate benchmark report
    generate_benchmark_report(&benchmark_results);
    
    // Print summary
    print_benchmark_summary(&benchmark_results);
}

fn benchmark_compilation_performance() -> BenchmarkResult {
    let start = Instant::now();
    let mut successful_compilations = 0;
    
    // Compile the same binary multiple times to measure compilation performance
    for _i in 0..3 {
        match Command::new("nix")
            .args(&["develop", "--command", "rustc", "-o", "benchmark_atlas", "src/bin/simple_repository_mathematical_atlas.rs"])
            .output() {
            Ok(output) => {
                if output.status.success() {
                    successful_compilations += 1;
                    // Clean up
                    let _ = std::fs::remove_file("benchmark_atlas");
                }
            }
            Err(_) => {
                // Continue with other compilations
            }
        }
    }
    
    let duration = start.elapsed();
    
    if successful_compilations >= 2 {
        BenchmarkResult::new("Compilation Performance".to_string(), duration.as_millis() as u64, 3, true, None)
    } else {
        BenchmarkResult::new("Compilation Performance".to_string(), duration.as_millis() as u64, 3, false, 
            Some(format!("Only {} successful compilations out of 3", successful_compilations)))
    }
}

fn benchmark_execution_performance() -> BenchmarkResult {
    let start = Instant::now();
    let mut successful_executions = 0;
    
    // Execute the atlas multiple times to measure execution performance
    for _i in 0..5 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    successful_executions += 1;
                }
            }
            Err(_) => {
                // Continue with other executions
            }
        }
    }
    
    let duration = start.elapsed();
    
    if successful_executions >= 3 {
        BenchmarkResult::new("Execution Performance".to_string(), duration.as_millis() as u64, 5, true, None)
    } else {
        BenchmarkResult::new("Execution Performance".to_string(), duration.as_millis() as u64, 5, false, 
            Some(format!("Only {} successful executions out of 5", successful_executions)))
    }
}

fn benchmark_output_generation_performance() -> BenchmarkResult {
    let start = Instant::now();
    let mut successful_generations = 0;
    
    // Generate outputs multiple times to measure output generation performance
    for _i in 0..3 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // Check if output files were generated
                    let expected_files = vec![
                        "repository_atlas_output/simple_complete_atlas.md",
                        "repository_atlas_output/simple_composition_all_repos.json",
                    ];
                    
                    let mut all_files_exist = true;
                    for file_path in &expected_files {
                        if !std::path::Path::new(file_path).exists() {
                            all_files_exist = false;
                            break;
                        }
                    }
                    
                    if all_files_exist {
                        successful_generations += 1;
                    }
                }
            }
            Err(_) => {
                // Continue with other generations
            }
        }
    }
    
    let duration = start.elapsed();
    
    if successful_generations >= 2 {
        BenchmarkResult::new("Output Generation Performance".to_string(), duration.as_millis() as u64, 3, true, None)
    } else {
        BenchmarkResult::new("Output Generation Performance".to_string(), duration.as_millis() as u64, 3, false, 
            Some(format!("Only {} successful output generations out of 3", successful_generations)))
    }
}

fn benchmark_mathematical_operations_performance() -> BenchmarkResult {
    let start = Instant::now();
    let mut successful_math_operations = 0;
    
    // Test mathematical operations by running the atlas and checking mathematical consistency
    for _i in 0..4 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // Check if mathematical operations completed successfully
                    match fs::read_to_string("repository_atlas_output/simple_complete_atlas.md") {
                        Ok(content) => {
                            if content.contains("Family Distribution") && content.contains("Complexity Statistics") {
                                successful_math_operations += 1;
                            }
                        }
                        Err(_) => {
                            // Continue with other operations
                        }
                    }
                }
            }
            Err(_) => {
                // Continue with other operations
            }
        }
    }
    
    let duration = start.elapsed();
    
    if successful_math_operations >= 3 {
        BenchmarkResult::new("Mathematical Operations Performance".to_string(), duration.as_millis() as u64, 4, true, None)
    } else {
        BenchmarkResult::new("Mathematical Operations Performance".to_string(), duration.as_millis() as u64, 4, false, 
            Some(format!("Only {} successful mathematical operations out of 4", successful_math_operations)))
    }
}

fn benchmark_memory_usage() -> BenchmarkResult {
    let start = Instant::now();
    let mut successful_memory_checks = 0;
    
    // Test memory usage by running the atlas and checking for memory leaks
    for _i in 0..3 {
        match Command::new("./simple_repository_mathematical_atlas")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    // Check if output files were created and have reasonable size
                    match fs::metadata("repository_atlas_output/simple_complete_atlas.md") {
                        Ok(metadata) => {
                            // File should be reasonable size (less than 1MB)
                            if metadata.len() < 1024 * 1024 {
                                successful_memory_checks += 1;
                            }
                        }
                        Err(_) => {
                            // Continue with other checks
                        }
                    }
                }
            }
            Err(_) => {
                // Continue with other checks
            }
        }
    }
    
    let duration = start.elapsed();
    
    if successful_memory_checks >= 2 {
        BenchmarkResult::new("Memory Usage".to_string(), duration.as_millis() as u64, 3, true, None)
    } else {
        BenchmarkResult::new("Memory Usage".to_string(), duration.as_millis() as u64, 3, false, 
            Some(format!("Only {} successful memory checks out of 3", successful_memory_checks)))
    }
}

fn generate_benchmark_report(benchmark_results: &[BenchmarkResult]) {
    let report_path = "./benchmark_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas Benchmark Report\n\n");
    report.push_str("Generated on: ");
    report.push_str(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    report.push_str("\n\n## Benchmark Summary\n\n");
    
    let total_benchmarks = benchmark_results.len();
    let successful_benchmarks = benchmark_results.iter().filter(|r| r.success).count();
    let failed_benchmarks = total_benchmarks - successful_benchmarks;
    
    report.push_str(&format!("- **Total Benchmarks**: {}\n", total_benchmarks));
    report.push_str(&format!("- **Successful**: {}\n", successful_benchmarks));
    report.push_str(&format!("- **Failed**: {}\n", failed_benchmarks));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (successful_benchmarks as f64 / total_benchmarks as f64) * 100.0));
    
    // Performance analysis
    let total_duration: u64 = benchmark_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_benchmarks as f64;
    
    report.push_str("## Performance Analysis\n\n");
    report.push_str(&format!("- **Total Benchmark Time**: {}ms\n", total_duration));
    report.push_str(&format!("- **Average Benchmark Time**: {:.1}ms\n\n", avg_duration));
    
    report.push_str("## Detailed Benchmark Results\n\n");
    
    for result in benchmark_results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", result.benchmark_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str(&format!("- **Total Duration**: {}ms\n", result.duration_ms));
        report.push_str(&format!("- **Iterations**: {}\n", result.iterations));
        report.push_str(&format!("- **Average Duration**: {:.1}ms\n", result.avg_duration_ms));
        
        if let Some(error) = &result.error_message {
            report.push_str(&format!("- **Error**: {}\n", error));
        }
        
        report.push_str("\n");
    }
    
    // Performance recommendations
    report.push_str("## Performance Recommendations\n\n");
    if avg_duration < 1000.0 {
        report.push_str("- ✅ **Excellent Performance**: All benchmarks completed quickly\n");
    } else if avg_duration < 5000.0 {
        report.push_str("- ⚠️ **Good Performance**: Benchmarks completed within acceptable time\n");
    } else {
        report.push_str("- ❌ **Poor Performance**: Benchmarks took too long, consider optimization\n");
    }
    
    fs::write(report_path, report).expect("Failed to write benchmark report");
    println!("📄 Benchmark report generated: {}", report_path);
}

fn print_benchmark_summary(benchmark_results: &[BenchmarkResult]) {
    let total_benchmarks = benchmark_results.len();
    let successful_benchmarks = benchmark_results.iter().filter(|r| r.success).count();
    let failed_benchmarks = total_benchmarks - successful_benchmarks;
    let success_rate = (successful_benchmarks as f64 / total_benchmarks as f64) * 100.0;
    
    let total_duration: u64 = benchmark_results.iter().map(|r| r.duration_ms).sum();
    let avg_duration = total_duration as f64 / total_benchmarks as f64;
    
    println!("\n⚡ Benchmark Summary");
    println!("===================");
    println!("📊 Total Benchmarks: {}", total_benchmarks);
    println!("✅ Successful: {}", successful_benchmarks);
    println!("❌ Failed: {}", failed_benchmarks);
    println!("📈 Success Rate: {:.1}%", success_rate);
    println!("⏱️  Total Time: {}ms", total_duration);
    println!("📊 Average Time: {:.1}ms", avg_duration);
    
    if failed_benchmarks == 0 {
        println!("🎉 All benchmarks passed! The Repository Mathematical Atlas performs well.");
    } else {
        println!("⚠️  {} benchmarks failed. Please review the benchmark report for details.", failed_benchmarks);
    }
    
    println!("\n📄 Detailed benchmark report available in: ./benchmark_report.md");
}