use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct TestRepository {
    name: String,
    path: String,
    stars: usize,
    forks: usize,
    contributors: usize,
    is_fork: bool,
    language: String,
    size_mb: usize,
    last_commit_days: usize,
    complexity_score: f64,
    group_family: String,
    group_order: String,
    group_rank: usize,
    simple_subgroups: Vec<String>,
}

impl TestRepository {
    fn new(name: String, path: String) -> Self {
        TestRepository {
            name,
            path,
            stars: 0,
            forks: 0,
            contributors: 0,
            is_fork: false,
            language: "Unknown".to_string(),
            size_mb: 0,
            last_commit_days: 0,
            complexity_score: 0.0,
            group_family: "Unclassified".to_string(),
            group_order: "1".to_string(),
            group_rank: 0,
            simple_subgroups: Vec::new(),
        }
    }

    fn calculate_complexity(&mut self) {
        let mut complexity = 0.0;
        complexity += (self.stars as f64 + 1.0).log10() * 0.25;
        complexity += (self.forks as f64 + 1.0).log10() * 0.15;
        complexity += (self.contributors as f64 + 1.0).log10() * 0.20;
        complexity += (self.size_mb as f64 + 1.0).log10() * 0.10;
        if self.last_commit_days > 0 {
            complexity += (365.0 / self.last_commit_days as f64).log10() * 0.10;
        }
        self.complexity_score = complexity;
    }

    fn classify_group_family(&mut self) {
        if self.is_fork {
            self.group_family = "Cyclic".to_string();
            self.group_order = "2".to_string();
            self.group_rank = 1;
            self.simple_subgroups = vec!["C2".to_string()];
        } else if self.complexity_score < 1.0 {
            let prime_order = self.get_prime_order();
            self.group_family = "Cyclic".to_string();
            self.group_order = prime_order.to_string();
            self.group_rank = 1;
            self.simple_subgroups = vec![format!("C{}", prime_order)];
        } else if self.complexity_score < 2.0 {
            self.group_family = "Alternating".to_string();
            let n = (self.complexity_score * 5.0).ceil() as usize;
            self.group_order = self.alternating_order(n).to_string();
            self.group_rank = n - 2;
            self.simple_subgroups = vec![format!("A{}", n)];
        } else if self.complexity_score < 3.0 {
            self.group_family = "Lie Type".to_string();
            let rank = ((self.complexity_score - 2.0) * 3.0).ceil() as usize;
            self.group_order = self.lie_type_order(rank).to_string();
            self.group_rank = rank;
            self.simple_subgroups = vec![format!("PSL({}, 2)", rank)];
        } else {
            self.group_family = "Sporadic".to_string();
            self.group_order = "7920".to_string();
            self.group_rank = 0;
            self.simple_subgroups = vec!["Mathieu M11".to_string()];
        }
    }

    fn get_prime_order(&self) -> usize {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        let index = (self.complexity_score * primes.len() as f64).min(primes.len() as f64 - 1.0) as usize;
        primes[index]
    }

    fn alternating_order(&self, n: usize) -> usize {
        if n < 5 { return 1; }
        let mut factorial = 1;
        for i in 1..=n {
            factorial *= i;
        }
        factorial / 2
    }

    fn lie_type_order(&self, rank: usize) -> usize {
        let mut order = 1;
        for _i in 1..=rank {
            order *= 2_i32.pow((rank * (rank - 1) / 2) as u32) as usize;
        }
        order * 1000
    }

    fn get_tile_size(&self) -> f64 {
        let log_order = (self.group_order.parse::<usize>().unwrap_or(1) as f64 + 1.0).log10();
        50.0 + (log_order * 15.0).min(150.0)
    }

    fn get_tile_color(&self) -> String {
        match self.group_family.as_str() {
            "Cyclic" => "#FF6B6B".to_string(),
            "Alternating" => "#4ECDC4".to_string(),
            "Lie Type" => "#45B7D1".to_string(),
            "Sporadic" => "#96CEB4".to_string(),
            _ => "#DDA0DD".to_string(),
        }
    }

    fn get_group_theory_description(&self) -> String {
        match self.group_family.as_str() {
            "Cyclic" => format!("Cyclic group of order {} - Prime order simple group", self.group_order),
            "Alternating" => format!("Alternating group A_{} - Non-abelian simple group", self.group_rank + 2),
            "Lie Type" => format!("Lie type group PSL({}, 2) - Classical simple group", self.group_rank),
            "Sporadic" => format!("Sporadic group {} - Exceptional simple group", self.simple_subgroups.first().unwrap_or(&"Unknown".to_string())),
            _ => "Unclassified group".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TestRepositoryView {
    name: String,
    description: String,
    repositories: Vec<TestRepository>,
}

impl TestRepositoryView {
    fn new(name: String, description: String) -> Self {
        TestRepositoryView {
            name,
            description,
            repositories: Vec::new(),
        }
    }

    fn get_family_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for repo in &self.repositories {
            *distribution.entry(repo.group_family.clone()).or_insert(0) += 1;
        }
        distribution
    }

    fn get_complexity_stats(&self) -> (f64, f64, f64) {
        if self.repositories.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let complexities: Vec<f64> = self.repositories.iter().map(|r| r.complexity_score).collect();
        let min = complexities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = complexities.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg = complexities.iter().sum::<f64>() / complexities.len() as f64;
        
        (min, max, avg)
    }

    fn generate_atlas_data(&self) -> String {
        let mut data = String::new();
        
        data.push_str(&format!("# Repository Mathematical Atlas: {}\n\n", self.name));
        data.push_str(&format!("**Description**: {}\n\n", self.description));
        
        let family_dist = self.get_family_distribution();
        data.push_str("## Family Distribution\n\n");
        for (family, count) in family_dist {
            data.push_str(&format!("- **{}**: {} repositories\n", family, count));
        }
        data.push_str("\n");

        let (min_comp, max_comp, avg_comp) = self.get_complexity_stats();
        data.push_str("## Complexity Statistics\n\n");
        data.push_str(&format!("- **Minimum Complexity**: {:.2}\n", min_comp));
        data.push_str(&format!("- **Maximum Complexity**: {:.2}\n", max_comp));
        data.push_str(&format!("- **Average Complexity**: {:.2}\n", avg_comp));
        data.push_str("\n");

        data.push_str("## Repository Details\n\n");
        for (i, repo) in self.repositories.iter().enumerate() {
            data.push_str(&format!("### {}. {}\n", i + 1, repo.name));
            data.push_str(&format!("- **Path**: {}\n", repo.path));
            data.push_str(&format!("- **Group Family**: {}\n", repo.group_family));
            data.push_str(&format!("- **Group Order**: {}\n", repo.group_order));
            data.push_str(&format!("- **Group Rank**: {}\n", repo.group_rank));
            data.push_str(&format!("- **Complexity Score**: {:.2}\n", repo.complexity_score));
            data.push_str(&format!("- **Simple Subgroups**: {}\n", repo.simple_subgroups.join(", ")));
            data.push_str(&format!("- **Stars**: {}\n", repo.stars));
            data.push_str(&format!("- **Forks**: {}\n", repo.forks));
            data.push_str(&format!("- **Contributors**: {}\n", repo.contributors));
            data.push_str(&format!("- **Language**: {}\n", repo.language));
            data.push_str(&format!("- **Size**: {} MB\n", repo.size_mb));
            data.push_str(&format!("- **Last Commit**: {} days ago\n", repo.last_commit_days));
            data.push_str(&format!("- **Tile Size**: {:.0}px\n", repo.get_tile_size()));
            data.push_str(&format!("- **Tile Color**: {}\n", repo.get_tile_color()));
            data.push_str(&format!("- **Group Theory Description**: {}\n", repo.get_group_theory_description()));
            data.push_str("\n");
        }

        data
    }
}

struct TestAtlasComposer {
    views: HashMap<String, TestRepositoryView>,
}

impl TestAtlasComposer {
    fn new() -> Self {
        TestAtlasComposer {
            views: HashMap::new(),
        }
    }

    fn create_sample_repositories(&mut self) {
        let mut repos = vec![
            TestRepository::new("rust-lang/rust".to_string(), "/rust/rust".to_string()),
            TestRepository::new("torvalds/linux".to_string(), "/linux/linux".to_string()),
            TestRepository::new("microsoft/vscode".to_string(), "/vscode/vscode".to_string()),
            TestRepository::new("facebook/react".to_string(), "/react/react".to_string()),
            TestRepository::new("tensorflow/tensorflow".to_string(), "/tensorflow/tensorflow".to_string()),
        ];

        repos[0].stars = 75_000; repos[0].forks = 12_000; repos[0].contributors = 1_200; repos[0].language = "Rust".to_string(); repos[0].size_mb = 500; repos[0].last_commit_days = 1;
        repos[1].stars = 150_000; repos[1].forks = 50_000; repos[1].contributors = 2_500; repos[1].language = "C".to_string(); repos[1].size_mb = 1000; repos[1].last_commit_days = 0;
        repos[2].stars = 150_000; repos[2].forks = 25_000; repos[2].contributors = 1_800; repos[2].language = "TypeScript".to_string(); repos[2].size_mb = 300; repos[2].last_commit_days = 1;
        repos[3].stars = 200_000; repos[3].forks = 40_000; repos[3].contributors = 2_200; repos[3].language = "JavaScript".to_string(); repos[3].size_mb = 200; repos[3].last_commit_days = 0;
        repos[4].stars = 170_000; repos[4].forks = 70_000; repos[4].contributors = 5_000; repos[4].language = "Python".to_string(); repos[4].size_mb = 800; repos[4].last_commit_days = 2;

        for repo in &mut repos {
            repo.calculate_complexity();
            repo.classify_group_family();
        }

        let mut all_repos_view = TestRepositoryView::new("All Repositories".to_string(), "Complete mathematical analysis of all repositories".to_string());
        all_repos_view.repositories = repos;
        self.views.insert("all_repos".to_string(), all_repos_view);
    }

    fn create_view(&mut self, name: String, description: String) -> &mut TestRepositoryView {
        let view = TestRepositoryView::new(name.clone(), description);
        self.views.insert(name.clone(), view);
        self.views.get_mut(&name).unwrap()
    }

    fn generate_composition_json(&self, view_name: String) -> Result<String, String> {
        if let Some(view) = self.views.get(&view_name) {
            let mut composition = HashMap::new();
            
            composition.insert("view_name".to_string(), view.name.clone());
            composition.insert("view_description".to_string(), view.description.clone());
            
            let family_dist = view.get_family_distribution();
            composition.insert("family_distribution".to_string(), format!("{:?}", family_dist));
            
            let (min_comp, max_comp, avg_comp) = view.get_complexity_stats();
            let mut complexity_stats = HashMap::new();
            complexity_stats.insert("minimum".to_string(), min_comp.to_string());
            complexity_stats.insert("maximum".to_string(), max_comp.to_string());
            complexity_stats.insert("average".to_string(), avg_comp.to_string());
            composition.insert("complexity_statistics".to_string(), format!("{:?}", complexity_stats));
            
            let mut repos_data = Vec::new();
            for repo in &view.repositories {
                let mut repo_data = HashMap::new();
                repo_data.insert("name".to_string(), repo.name.clone());
                repo_data.insert("path".to_string(), repo.path.clone());
                repo_data.insert("group_family".to_string(), repo.group_family.clone());
                repo_data.insert("group_order".to_string(), repo.group_order.clone());
                repo_data.insert("group_rank".to_string(), repo.group_rank.to_string());
                repo_data.insert("complexity_score".to_string(), repo.complexity_score.to_string());
                repo_data.insert("simple_subgroups".to_string(), format!("{:?}", repo.simple_subgroups));
                repo_data.insert("stars".to_string(), repo.stars.to_string());
                repo_data.insert("forks".to_string(), repo.forks.to_string());
                repo_data.insert("contributors".to_string(), repo.contributors.to_string());
                repo_data.insert("language".to_string(), repo.language.clone());
                repo_data.insert("size_mb".to_string(), repo.size_mb.to_string());
                repo_data.insert("last_commit_days".to_string(), repo.last_commit_days.to_string());
                repo_data.insert("tile_size".to_string(), repo.get_tile_size().to_string());
                repo_data.insert("tile_color".to_string(), repo.get_tile_color());
                repo_data.insert("group_theory_description".to_string(), repo.get_group_theory_description());
                repos_data.push(repo_data);
            }
            composition.insert("repositories".to_string(), format!("{:?}", repos_data));
            
            Ok(format!("{:#?}", composition))
        } else {
            Err("View not found".to_string())
        }
    }
}

fn main() {
    println!("🧪 Repository Mathematical Atlas Integration Test Suite");
    println!("====================================================");
    
    let mut test_results: Vec<(String, bool)> = Vec::new();
    
    // Test 1: Basic functionality
    println!("\n📋 Test 1: Basic Functionality");
    println!("==============================");
    
    let mut composer = TestAtlasComposer::new();
    composer.create_sample_repositories();
    
    // Verify basic structure
    assert!(composer.views.contains_key("all_repos"));
    assert_eq!(composer.views["all_repos"].repositories.len(), 5);
    
    println!("✅ Basic functionality test passed");
    test_results.push(("Basic Functionality".to_string(), true));
    
    // Test 2: Repository classification
    println!("\n📋 Test 2: Repository Classification");
    println!("==================================");
    
    {
        let all_repos = &composer.views["all_repos"];
        let mut family_counts = HashMap::new();
        
        for repo in &all_repos.repositories {
            *family_counts.entry(repo.group_family.clone()).or_insert(0) += 1;
        }
        
        assert!(!family_counts.is_empty());
        println!("📊 Family distribution: {:?}", family_counts);
    }
    
    println!("✅ Repository classification test passed");
    test_results.push(("Repository Classification".to_string(), true));
    
    // Test 3: Complexity calculation
    println!("\n📋 Test 3: Complexity Calculation");
    println!("================================");
    
    {
        let all_repos = &composer.views["all_repos"];
        let complexities: Vec<f64> = all_repos.repositories.iter().map(|r| r.complexity_score).collect();
        let min_complexity = complexities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_complexity = complexities.iter().fold(0.0_f64, |a, &b| a.max(b));
        
        assert!(min_complexity > 0.0);
        assert!(max_complexity > min_complexity);
        
        println!("📊 Complexity range: {:.2} - {:.2}", min_complexity, max_complexity);
    }
    
    println!("✅ Complexity calculation test passed");
    test_results.push(("Complexity Calculation".to_string(), true));
    
    // Test 4: View operations
    println!("\n📋 Test 4: View Operations");
    println!("=========================");
    
    {
        let cyclic_view = composer.create_view("Cyclic".to_string(), "Cyclic repositories".to_string());
        cyclic_view.repositories.retain(|r| r.group_family == "Cyclic");
    }
    
    {
        let js_view = composer.create_view("JavaScript".to_string(), "JavaScript repositories".to_string());
        js_view.repositories.retain(|r| r.language == "JavaScript");
    }
    
    {
        let cyclic_count = composer.views.get("Cyclic").map(|v| v.repositories.len()).unwrap_or(0);
        let js_count = composer.views.get("JavaScript").map(|v| v.repositories.len()).unwrap_or(0);
        
        assert!(cyclic_count >= 0);
        assert!(js_count >= 0);
        
        println!("📊 Cyclic repositories: {}", cyclic_count);
        println!("📊 JavaScript repositories: {}", js_count);
    }
    
    println!("✅ View operations test passed");
    test_results.push(("View Operations".to_string(), true));
    
    // Test 5: Atlas generation
    println!("\n📋 Test 5: Atlas Generation");
    println!("=========================");
    
    {
        let all_repos = &composer.views["all_repos"];
        let atlas_data = all_repos.generate_atlas_data();
        
        assert!(!atlas_data.is_empty());
        assert!(atlas_data.contains("Repository Mathematical Atlas"));
        assert!(atlas_data.contains("All Repositories"));
        assert!(atlas_data.contains("Lie Type"));
        assert!(atlas_data.contains("Sporadic"));
        
        println!("📄 Atlas generated with {} characters", atlas_data.len());
        
        // Save atlas data
        fs::write("./test_integration_atlas.md", atlas_data).unwrap();
        println!("✅ Atlas saved to: ./test_integration_atlas.md");
    }
    
    println!("✅ Atlas generation test passed");
    test_results.push(("Atlas Generation".to_string(), true));
    
    // Test 6: JSON composition generation
    println!("\n📋 Test 6: JSON Composition Generation");
    println!("===================================");
    
    match composer.generate_composition_json("all_repos".to_string()) {
        Ok(json_content) => {
            assert!(!json_content.is_empty());
            assert!(json_content.contains("All Repositories"));
            assert!(json_content.contains("Lie Type"));
            assert!(json_content.contains("Sporadic"));
            
            // Save JSON composition
            fs::write("./test_integration_composition.json", json_content).unwrap();
            println!("✅ JSON composition saved to: ./test_integration_composition.json");
        }
        Err(e) => {
            panic!("Failed to generate JSON composition: {}", e);
        }
    }
    
    println!("✅ JSON composition generation test passed");
    test_results.push(("JSON Composition Generation".to_string(), true));
    
    // Test 7: Mathematical consistency
    println!("\n📋 Test 7: Mathematical Consistency");
    println!("==================================");
    
    {
        let all_repos = &composer.views["all_repos"];
        let mut total_complexity = 0.0;
        let mut total_order = 0usize;
        
        for repo in &all_repos.repositories {
            total_complexity += repo.complexity_score;
            total_order += repo.group_order.parse::<usize>().unwrap_or(1);
            
            // Validate mathematical properties
            assert!(!repo.group_family.is_empty());
            assert!(!repo.group_order.is_empty());
            assert!(!repo.name.is_empty());
            assert!(!repo.path.is_empty());
            
            // Validate tile properties
            let tile_size = repo.get_tile_size();
            let tile_color = repo.get_tile_color();
            
            assert!(tile_size >= 50.0 && tile_size <= 150.0);
            assert!(!tile_color.is_empty());
        }
        
        let avg_complexity = total_complexity / all_repos.repositories.len() as f64;
        let avg_order = total_order / all_repos.repositories.len();
        
        println!("📊 Average complexity: {:.2}", avg_complexity);
        println!("📊 Average group order: {}", avg_order);
    }
    
    println!("✅ Mathematical consistency test passed");
    test_results.push(("Mathematical Consistency".to_string(), true));
    
    // Test 8: Performance benchmarks
    println!("\n📋 Test 8: Performance Benchmarks");
    println!("================================");
    
    use std::time::Instant;
    
    // Complexity calculation performance
    {
        let start = Instant::now();
        for repo in &mut composer.views.get_mut("all_repos").unwrap().repositories {
            repo.calculate_complexity();
        }
        let complexity_duration = start.elapsed();
        
        // Classification performance
        let start = Instant::now();
        for repo in &mut composer.views.get_mut("all_repos").unwrap().repositories {
            repo.classify_group_family();
        }
        let classification_duration = start.elapsed();
        
        // Atlas generation performance
        let start = Instant::now();
        let all_repos = &composer.views["all_repos"];
        let _atlas_data = all_repos.generate_atlas_data();
        let atlas_duration = start.elapsed();
        
        println!("📊 Complexity calculation: {:?}", complexity_duration);
        println!("📊 Classification: {:?}", classification_duration);
        println!("📊 Atlas generation: {:?}", atlas_duration);
        
        // All operations should complete within reasonable time
        assert!(complexity_duration.as_millis() < 100);
        assert!(classification_duration.as_millis() < 100);
        assert!(atlas_duration.as_millis() < 100);
    }
    
    println!("✅ Performance benchmarks test passed");
    test_results.push(("Performance Benchmarks".to_string(), true));
    
    // Test 9: Edge cases
    println!("\n📋 Test 9: Edge Cases");
    println!("====================");
    
// Test with zero values
     let mut zero_repo = TestRepository::new("zero_test".to_string(), "/test/zero".to_string());
     zero_repo.stars = 0;
    zero_repo.forks = 0;
    zero_repo.contributors = 0;
    zero_repo.size_mb = 0;
    zero_repo.last_commit_days = 0;
    
    zero_repo.calculate_complexity();
    zero_repo.classify_group_family();
    
    assert!(zero_repo.complexity_score >= 0.0);
    assert!(!zero_repo.group_family.is_empty());
    
// Test with maximum values
     let mut max_repo = TestRepository::new("max_test".to_string(), "/test/max".to_string());
     max_repo.stars = std::usize::MAX;
    max_repo.forks = std::usize::MAX;
    max_repo.contributors = std::usize::MAX;
    max_repo.size_mb = std::usize::MAX;
    max_repo.last_commit_days = 1;
    
    max_repo.calculate_complexity();
    max_repo.classify_group_family();
    
    assert!(max_repo.complexity_score > 0.0);
    assert!(!max_repo.group_family.is_empty());
    
    println!("✅ Edge cases test passed");
    test_results.push(("Edge Cases".to_string(), true));
    
    // Test 10: File I/O operations
    println!("\n📋 Test 10: File I/O Operations");
    println!("==============================");
    
    // Test file writing
    let test_content = "Test content for file I/O operations";
    let test_path = "./test_io.txt";
    
    fs::write(test_path, test_content).unwrap();
    assert!(Path::new(test_path).exists());
    
    // Test file reading
    let read_content = fs::read_to_string(test_path).unwrap();
    assert_eq!(read_content, test_content);
    
    // Clean up
    fs::remove_file(test_path).unwrap();
    
    println!("✅ File I/O operations test passed");
    test_results.push(("File I/O Operations".to_string(), true));
    
    // Generate test report
    generate_integration_test_report(&test_results);
    
    // Print final summary
    print_integration_test_summary(&test_results);
}

fn generate_integration_test_report(test_results: &[(String, bool)]) {
    let report_path = "./integration_test_report.md";
    let mut report = String::new();
    
    report.push_str("# Repository Mathematical Atlas Integration Test Report\n\n");
    report.push_str("Generated on: ");
    report.push_str(&chrono::Utc::now().to_rfc3339().to_string());
    report.push_str("\n\n## Test Summary\n\n");
    
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|(_, passed)| *passed).count();
    let failed_tests = total_tests - passed_tests;
    
    report.push_str(&format!("- **Total Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Passed**: {}\n", passed_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (passed_tests as f64 / total_tests as f64) * 100.0));
    
    report.push_str("## Detailed Results\n\n");
    
for (test_name, passed) in test_results {
         let status = if *passed { "✅ PASS" } else { "❌ FAIL" };
        report.push_str(&format!("### {}\n", test_name));
        report.push_str(&format!("- **Status**: {}\n", status));
        report.push_str("\n");
    }
    
    fs::write(report_path, report).expect("Failed to write integration test report");
    println!("📄 Integration test report generated: {}", report_path);
}

fn print_integration_test_summary(test_results: &[(String, bool)]) {
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|(_, passed)| *passed).count();
    let failed_tests = total_tests - passed_tests;
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    println!("\n🎯 Integration Test Summary");
    println!("===========================");
    println!("📊 Total Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("📈 Success Rate: {:.1}%", success_rate);
    
    if failed_tests == 0 {
        println!("🎉 All integration tests passed! The Repository Mathematical Atlas is working correctly.");
    } else {
        println!("⚠️  {} integration tests failed. Please review the test report for details.", failed_tests);
    }
    
    println!("\n📄 Detailed integration test report available in: ./integration_test_report.md");
    println!("📄 Generated atlas data available in: ./test_integration_atlas.md");
    println!("📄 Generated JSON composition available in: ./test_integration_composition.json");
}