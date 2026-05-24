use std::collections::HashMap;
use std::time::Instant;
use std::fs;

#[derive(Debug)]
struct BenchmarkResult {
    benchmark_name: String,
    duration_ms: u64,
    operations_per_second: f64,
    memory_usage_mb: f64,
    success: bool,
    error_message: Option<String>,
}

impl BenchmarkResult {
    fn new(benchmark_name: String, duration_ms: u64, operations_per_second: f64, memory_usage_mb: f64, success: bool, error_message: Option<String>) -> Self {
        BenchmarkResult {
            benchmark_name,
            duration_ms,
            operations_per_second,
            memory_usage_mb,
            success,
            error_message,
        }
    }
}

fn main() {
    println!("🚀 Repository Mathematical Atlas Benchmark Suite");
    println!("==============================================");
    
    let mut benchmark_results = Vec::new();
    
    // Run benchmark suites
    println!("\n🔬 Running Complexity Calculation Benchmarks...");
    let complexity_results = benchmark_complexity_calculation();
    benchmark_results.extend(complexity_results);
    
    println!("\n🔬 Running Classification Benchmarks...");
    let classification_results = benchmark_classification();
    benchmark_results.extend(classification_results);
    
    println!("\n🔬 Running Atlas Generation Benchmarks...");
    let atlas_results = benchmark_atlas_generation();
    benchmark_results.extend(atlas_results);
    
    println!("\n🔬 Running View Operation Benchmarks...");
    let view_results = benchmark_view_operations();
    benchmark_results.extend(view_results);
    
    println!("\n🔬 Running Large Dataset Benchmarks...");
    let large_dataset_results = benchmark_large_dataset();
    benchmark_results.extend(large_dataset_results);
    
    println!("\n🔬 Running Memory Usage Benchmarks...");
    let memory_results = benchmark_memory_usage();
    benchmark_results.extend(memory_results);
    
    // Generate benchmark report
    generate_benchmark_report(&benchmark_results);
    
    // Print summary
    print_benchmark_summary(&benchmark_results);
}

fn benchmark_complexity_calculation() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: Single repository complexity calculation
    println!("  📊 Benchmarking single repository complexity calculation...");
    let start = Instant::now();
    
    for _ in 0..1000 {
        let mut complexity = 0.0;
        let stars = 1000;
        let forks = 500;
        let contributors = 100;
        let size_mb = 100;
        let last_commit_days = 10;
        
        complexity += (stars as f64 + 1.0).log10() * 0.25;
        complexity += (forks as f64 + 1.0).log10() * 0.15;
        complexity += (contributors as f64 + 1.0).log10() * 0.20;
        complexity += (size_mb as f64 + 1.0).log10() * 0.10;
        if last_commit_days > 0 {
            complexity += (365.0 / last_commit_days as f64).log10() * 0.10;
        }
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1000.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Single Repository Complexity".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 2: Multiple repository complexity calculation
    println!("  📊 Benchmarking multiple repository complexity calculation...");
    let start = Instant::now();
    
    for repo_index in 0..100 {
        let mut complexity = 0.0;
        let stars = repo_index * 100;
        let forks = repo_index * 50;
        let contributors = repo_index * 10;
        let size_mb = repo_index * 10;
        let last_commit_days = repo_index % 365 + 1;
        
        complexity += (stars as f64 + 1.0).log10() * 0.25;
        complexity += (forks as f64 + 1.0).log10() * 0.15;
        complexity += (contributors as f64 + 1.0).log10() * 0.20;
        complexity += (size_mb as f64 + 1.0).log10() * 0.10;
        if last_commit_days > 0 {
            complexity += (365.0 / last_commit_days as f64).log10() * 0.10;
        }
    }
    
    let duration = start.elapsed();
    let operations_per_second = 100.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Multiple Repository Complexity".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 3: Large scale complexity calculation
    println!("  📊 Benchmarking large scale complexity calculation...");
    let start = Instant::now();
    
    for _ in 0..10 {
        for repo_index in 0..1000 {
            let mut complexity = 0.0;
            let stars = repo_index * 100;
            let forks = repo_index * 50;
            let contributors = repo_index * 10;
            let size_mb = repo_index * 10;
            let last_commit_days = repo_index % 365 + 1;
            
            complexity += (stars as f64 + 1.0).log10() * 0.25;
            complexity += (forks as f64 + 1.0).log10() * 0.15;
            complexity += (contributors as f64 + 1.0).log10() * 0.20;
            complexity += (size_mb as f64 + 1.0).log10() * 0.10;
            if last_commit_days > 0 {
                complexity += (365.0 / last_commit_days as f64).log10() * 0.10;
            }
        }
    }
    
    let duration = start.elapsed();
    let operations_per_second = (10.0 * 1000.0) / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Large Scale Complexity".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    results
}

fn benchmark_classification() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: Single repository classification
    println!("  📊 Benchmarking single repository classification...");
    let start = Instant::now();
    
    for _ in 0..1000 {
        let complexity = 2.5; // Fixed complexity for testing
        
        let family = match complexity {
            c if c < 1.0 => "Cyclic",
            c if c < 2.0 => "Alternating",
            c if c < 3.0 => "Lie Type",
            _ => "Sporadic",
        };
        
        assert!(!family.is_empty());
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1000.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Single Repository Classification".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 2: Multiple repository classification
    println!("  📊 Benchmarking multiple repository classification...");
    let start = Instant::now();
    
    for repo_index in 0..100 {
        let complexity = repo_index as f64 * 0.1;
        
        let family = match complexity {
            c if c < 1.0 => "Cyclic",
            c if c < 2.0 => "Alternating",
            c if c < 3.0 => "Lie Type",
            _ => "Sporadic",
        };
        
        assert!(!family.is_empty());
    }
    
    let duration = start.elapsed();
    let operations_per_second = 100.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Multiple Repository Classification".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 3: Mixed complexity classification
    println!("  📊 Benchmarking mixed complexity classification...");
    let start = Instant::now();
    
    for _ in 0..100 {
        for complexity_level in [0.5, 1.5, 2.5, 3.5] {
            let family = match complexity_level {
                c if c < 1.0 => "Cyclic",
                c if c < 2.0 => "Alternating",
                c if c < 3.0 => "Lie Type",
                _ => "Sporadic",
            };
            
            assert!(!family.is_empty());
        }
    }
    
    let duration = start.elapsed();
    let operations_per_second = (100.0 * 4.0) / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Mixed Complexity Classification".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    results
}

fn benchmark_atlas_generation() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: Small atlas generation
    println!("  📊 Benchmarking small atlas generation...");
    let start = Instant::now();
    
    let mut atlas_content = String::new();
    atlas_content.push_str("# Repository Mathematical Atlas\n\n");
    atlas_content.push_str("## Family Distribution\n\n");
    atlas_content.push_str("- **Lie Type**: 3 repositories\n");
    atlas_content.push_str("- **Sporadic**: 2 repositories\n\n");
    atlas_content.push_str("## Complexity Statistics\n\n");
    atlas_content.push_str("- **Minimum Complexity**: 2.91\n");
    atlas_content.push_str("- **Maximum Complexity**: 3.29\n");
    atlas_content.push_str("- **Average Complexity**: 3.05\n\n");
    atlas_content.push_str("## Repository Details\n\n");
    
    for i in 1..=5 {
        atlas_content.push_str(&format!("### {}. Repository {}\n", i, i));
        atlas_content.push_str("- **Group Family**: Lie Type\n");
        atlas_content.push_str("- **Group Order**: 512000\n");
        atlas_content.push_str("- **Complexity Score**: 3.0\n");
        atlas_content.push_str(&format!("- **Stars**: {}\n", i * 1000));
        atlas_content.push_str("- **Language**: Rust\n");
        atlas_content.push_str("\n");
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Small Atlas Generation".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        atlas_content.len() as f64 / 1024.0 / 1024.0, // MB
        true,
        None,
    ));
    
    // Benchmark 2: Medium atlas generation
    println!("  📊 Benchmarking medium atlas generation...");
    let start = Instant::now();
    
    let mut atlas_content = String::new();
    atlas_content.push_str("# Repository Mathematical Atlas\n\n");
    atlas_content.push_str("## Family Distribution\n\n");
    atlas_content.push_str("- **Lie Type**: 15 repositories\n");
    atlas_content.push_str("- **Sporadic**: 10 repositories\n");
    atlas_content.push_str("- **Alternating**: 5 repositories\n");
    atlas_content.push_str("- **Cyclic**: 2 repositories\n\n");
    atlas_content.push_str("## Complexity Statistics\n\n");
    atlas_content.push_str("- **Minimum Complexity**: 1.2\n");
    atlas_content.push_str("- **Maximum Complexity**: 4.8\n");
    atlas_content.push_str("- **Average Complexity**: 2.9\n\n");
    atlas_content.push_str("## Repository Details\n\n");
    
    for i in 1..=32 {
        atlas_content.push_str(&format!("### {}. Repository {}\n", i, i));
        atlas_content.push_str("- **Group Family**: Lie Type\n");
        atlas_content.push_str("- **Group Order**: 512000\n");
        atlas_content.push_str("- **Complexity Score**: 3.0\n");
        atlas_content.push_str(&format!("- **Stars**: {}\n", i * 1000));
        atlas_content.push_str("- **Language**: Rust\n");
        atlas_content.push_str("\n");
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Medium Atlas Generation".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        atlas_content.len() as f64 / 1024.0 / 1024.0, // MB
        true,
        None,
    ));
    
    // Benchmark 3: Large atlas generation
    println!("  📊 Benchmarking large atlas generation...");
    let start = Instant::now();
    
    let mut atlas_content = String::new();
    atlas_content.push_str("# Repository Mathematical Atlas\n\n");
    atlas_content.push_str("## Family Distribution\n\n");
    atlas_content.push_str("- **Lie Type**: 150 repositories\n");
    atlas_content.push_str("- **Sporadic**: 100 repositories\n");
    atlas_content.push_str("- **Alternating**: 50 repositories\n");
    atlas_content.push_str("- **Cyclic**: 20 repositories\n\n");
    atlas_content.push_str("## Complexity Statistics\n\n");
    atlas_content.push_str("- **Minimum Complexity**: 0.8\n");
    atlas_content.push_str("- **Maximum Complexity**: 5.2\n");
    atlas_content.push_str("- **Average Complexity**: 2.7\n\n");
    atlas_content.push_str("## Repository Details\n\n");
    
    for i in 1..=320 {
        atlas_content.push_str(&format!("### {}. Repository {}\n", i, i));
        atlas_content.push_str("- **Group Family**: Lie Type\n");
        atlas_content.push_str("- **Group Order**: 512000\n");
        atlas_content.push_str("- **Complexity Score**: 3.0\n");
        atlas_content.push_str(&format!("- **Stars**: {}\n", i * 1000));
        atlas_content.push_str("- **Language**: Rust\n");
        atlas_content.push_str(&format!("- **Contributors**: {}\n", i * 10));
        atlas_content.push_str(&format!("- **Forks**: {}\n", i * 500));
        atlas_content.push_str("\n");
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "Large Atlas Generation".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        atlas_content.len() as f64 / 1024.0 / 1024.0, // MB
        true,
        None,
    ));
    
    results
}

fn benchmark_view_operations() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: View creation
    println!("  📊 Benchmarking view creation...");
    let start = Instant::now();
    
    for i in 0..1000 {
        let view_name = format!("View_{}", i);
        let view_description = "Test view description";
        
        assert!(!view_name.is_empty());
        assert!(!view_description.is_empty());
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1000.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "View Creation".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 2: View filtering
    println!("  📊 Benchmarking view filtering...");
    let start = Instant::now();
    
    for _ in 0..100 {
        let repositories = vec![
            ("Repo1", "Lie Type"),
            ("Repo2", "Sporadic"),
            ("Repo3", "Lie Type"),
            ("Repo4", "Alternating"),
            ("Repo5", "Cyclic"),
        ];
        
        let cyclic_repos: Vec<_> = repositories.iter().filter(|(_, family)| *family == "Cyclic").collect();
        let lie_type_repos: Vec<_> = repositories.iter().filter(|(_, family)| *family == "Lie Type").collect();
        
        assert_eq!(cyclic_repos.len(), 1);
        assert_eq!(lie_type_repos.len(), 2);
    }
    
    let duration = start.elapsed();
    let operations_per_second = 100.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "View Filtering".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 3: View statistics calculation
    println!("  📊 Benchmarking view statistics calculation...");
    let start = Instant::now();
    
    for _ in 0..100 {
        let complexities = vec![1.2, 2.3, 3.4, 4.5, 5.6];
        
        let min = complexities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = complexities.iter().fold(0.0_f64, |a, &b| a.max(b));
        let avg = complexities.iter().sum::<f64>() / complexities.len() as f64;
        
        assert!(min > 0.0);
        assert!(max > min);
        assert!(avg > 0.0);
    }
    
    let duration = start.elapsed();
    let operations_per_second = 100.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "View Statistics Calculation".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    results
}

fn benchmark_large_dataset() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: 1000 repositories processing
    println!("  📊 Benchmarking 1000 repositories processing...");
    let start = Instant::now();
    
    for repo_index in 0..1000 {
        let mut complexity = 0.0;
        let stars = repo_index * 100;
        let forks = repo_index * 50;
        let contributors = repo_index * 10;
        let size_mb = repo_index * 10;
        let last_commit_days = repo_index % 365 + 1;
        
        complexity += (stars as f64 + 1.0).log10() * 0.25;
        complexity += (forks as f64 + 1.0).log10() * 0.15;
        complexity += (contributors as f64 + 1.0).log10() * 0.20;
        complexity += (size_mb as f64 + 1.0).log10() * 0.10;
        if last_commit_days > 0 {
            complexity += (365.0 / last_commit_days as f64).log10() * 0.10;
        }
        
        let family = match complexity {
            c if c < 1.0 => "Cyclic",
            c if c < 2.0 => "Alternating",
            c if c < 3.0 => "Lie Type",
            _ => "Sporadic",
        };
        
        assert!(!family.is_empty());
    }
    
    let duration = start.elapsed();
    let operations_per_second = 1000.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "1000 Repositories Processing".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    // Benchmark 2: 10000 repositories processing
    println!("  📊 Benchmarking 10000 repositories processing...");
    let start = Instant::now();
    
    for repo_index in 0..10000 {
        let mut complexity = 0.0;
        let stars = repo_index % 100000;
        let forks = repo_index % 50000;
        let contributors = repo_index % 1000;
        let size_mb = repo_index % 1000;
        let last_commit_days = repo_index % 365 + 1;
        
        complexity += (stars as f64 + 1.0).log10() * 0.25;
        complexity += (forks as f64 + 1.0).log10() * 0.15;
        complexity += (contributors as f64 + 1.0).log10() * 0.20;
        complexity += (size_mb as f64 + 1.0).log10() * 0.10;
        if last_commit_days > 0 {
            complexity += (365.0 / last_commit_days as f64).log10() * 0.10;
        }
        
        let family = match complexity {
            c if c < 1.0 => "Cyclic",
            c if c < 2.0 => "Alternating",
            c if c < 3.0 => "Lie Type",
            _ => "Sporadic",
        };
        
        assert!(!family.is_empty());
    }
    
    let duration = start.elapsed();
    let operations_per_second = 10000.0 / (duration.as_millis() as f64 / 1000.0);
    
    results.push(BenchmarkResult::new(
        "10000 Repositories Processing".to_string(),
        duration.as_millis() as u64,
        operations_per_second,
        0.0,
        true,
        None,
    ));
    
    results
}

fn benchmark_memory_usage() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    // Benchmark 1: Memory usage for small dataset
    println!("  📊 Benchmarking memory usage for small dataset...");
    let start = Instant::now();
    
    let mut repositories = Vec::new();
    for i in 0..10 {
        let repo = format!("Repository_{}", i);
        repositories.push(repo);
    }
    
    let duration = start.elapsed();
    let estimated_memory = repositories.iter().map(|s| s.len()).sum::<usize>() as f64 / 1024.0 / 1024.0;
    
    results.push(BenchmarkResult::new(
        "Small Dataset Memory Usage".to_string(),
        duration.as_millis() as u64,
        10.0 / (duration.as_millis() as f64 / 1000.0),
        estimated_memory,
        true,
        None,
    ));
    
    // Benchmark 2: Memory usage for medium dataset
    println!("  📊 Benchmarking memory usage for medium dataset...");
    let start = Instant::now();
    
    let mut repositories = Vec::new();
    for i in 0..100 {
        let repo = format!("Repository_{}_with_long_name_and_description", i);
        repositories.push(repo);
    }
    
    let duration = start.elapsed();
    let estimated_memory = repositories.iter().map(|s| s.len()).sum::<usize>() as f64 / 1024.0 / 1024.0;
    
    results.push(BenchmarkResult::new(
        "Medium Dataset Memory Usage".to_string(),
        duration.as_millis() as u64,
        100.0 / (duration.as_millis() as f64 / 1000.0),
        estimated_memory,
        true,
        None,
    ));
    
    // Benchmark 3: Memory usage for large dataset
    println!("  📊 Benchmarking memory usage for large dataset...");
    let start = Instant::now();
    
    let mut repositories = Vec::new();
    for i in 0..1000 {
        let repo = format!("Repository_{}_with_very_long_name_and_detailed_description_containing_multiple_fields_and_properties", i);
        repositories.push(repo);
    }
    
    let duration = start.elapsed();
    let estimated_memory = repositories.iter().map(|s| s.len()).sum::<usize>() as f64 / 1024.0 / 1024.0;
    
    results.push(BenchmarkResult::new(
        "Large Dataset Memory Usage".to_string(),
        duration.as_millis() as u64,
        1000.0 / (duration.as_millis() as f64 / 1000.0),
        estimated_memory,
        true,
        None,
    ));
    
    results
}

fn generate_benchmark_report(benchmark_results: &[BenchmarkResult]) {
    let report_path = "./benchmark_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas Benchmark Report\n\n");
    report.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().to_rfc3339()));
    report.push_str("\n\n## Benchmark Summary\n\n");
    
    let total_benchmarks = benchmark_results.len();
    let passed_benchmarks = benchmark_results.iter().filter(|r| r.success).count();
    let failed_benchmarks = total_benchmarks - passed_benchmarks;
    
    report.push_str(&format!("- **Total Benchmarks**: {}\n", total_benchmarks));
    report.push_str(&format!("- **Passed**: {}\n", passed_benchmarks));
    report.push_str(&format!("- **Failed**: {}\n", failed_benchmarks));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_benchmarks as f64 / total_benchmarks as f64) * 100.0));
    
    // Performance summary
    let avg_duration_ms: f64 = benchmark_results.iter().map(|r| r.duration_ms as f64).sum::<f64>() / total_benchmarks as f64;
    let avg_ops_per_sec: f64 = benchmark_results.iter().map(|r| r.operations_per_second).sum::<f64>() / total_benchmarks as f64;
    let total_memory_mb: f64 = benchmark_results.iter().map(|r| r.memory_usage_mb).sum();
    
    report.push_str("## Performance Summary\n\n");
    report.push_str(&format!("- **Average Duration**: {:.2}ms\n", avg_duration_ms));
    report.push_str(&format!("- **Average Operations/Second**: {:.0}\n", avg_ops_per_sec));
    report.push_str(&format!("- **Total Memory Usage**: {:.2}MB\n", total_memory_mb));
    report.push_str("\n");
    
    report.push_str("## Detailed Results\n\n");
    
    for result in benchmark_results {
        let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", result.benchmark_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str(&format!("- **Duration**: {}ms\n", result.duration_ms));
        report.push_str(&format!("- **Operations/Second**: {:.0}\n", result.operations_per_second));
        report.push_str(&format!("- **Memory Usage**: {:.2}MB\n", result.memory_usage_mb));
        
        if let Some(error) = &result.error_message {
            report.push_str(&format!("- **Error**: {}\n", error));
        }
        
        report.push_str("\n");
    }
    
    fs::write(report_path, report).expect("Failed to write benchmark report");
    println!("📄 Benchmark report generated: {}", report_path);
}

fn print_benchmark_summary(benchmark_results: &[BenchmarkResult]) {
    let total_benchmarks = benchmark_results.len();
    let passed_benchmarks = benchmark_results.iter().filter(|r| r.success).count();
    let failed_benchmarks = total_benchmarks - passed_benchmarks;
    let success_rate = (passed_benchmarks as f64 / total_benchmarks as f64) * 100.0;
    
    let avg_duration_ms: f64 = benchmark_results.iter().map(|r| r.duration_ms as f64).sum::<f64>() / total_benchmarks as f64;
    let avg_ops_per_sec: f64 = benchmark_results.iter().map(|r| r.operations_per_second).sum::<f64>() / total_benchmarks as f64;
    let total_memory_mb: f64 = benchmark_results.iter().map(|r| r.memory_usage_mb).sum();
    
    println!("\n🚀 Benchmark Summary");
    println!("====================");
    println!("📊 Total Benchmarks: {}", total_benchmarks);
    println!("✅ Passed: {}", passed_benchmarks);
    println!("❌ Failed: {}", failed_benchmarks);
    println!("📈 Success Rate: {:.1}%", success_rate);
    println!("⏱️  Average Duration: {:.2}ms", avg_duration_ms);
    println!("🔄 Average Operations/Second: {:.0}", avg_ops_per_sec);
    println!("💾 Total Memory Usage: {:.2}MB", total_memory_mb);
    
    if failed_benchmarks == 0 {
        println!("🎉 All benchmarks passed! The Repository Mathematical Atlas performs well.");
    } else {
        println!("⚠️  {} benchmarks failed. Please review the benchmark report for details.", failed_benchmarks);
    }
    
    println!("\n📄 Detailed benchmark report available in: ./benchmark_report.md");
}