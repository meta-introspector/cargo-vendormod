use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct ZkperfMetrics {
    coverage_score: f64,
    performance_score: f64,
    test_success_rate: f64,
    test_duration_ms: u64,
    files_generated: usize,
    test_categories_passed: usize,
    test_categories_failed: usize,
    coverage_analysis_status: String,
    performance_analysis_status: String,
    integration_status: String,
}

impl ZkperfMetrics {
    fn new() -> Self {
        ZkperfMetrics {
            coverage_score: 0.0,
            performance_score: 0.0,
            test_success_rate: 0.0,
            test_duration_ms: 0,
            files_generated: 0,
            test_categories_passed: 0,
            test_categories_failed: 0,
            coverage_analysis_status: "Not Tested".to_string(),
            performance_analysis_status: "Not Tested".to_string(),
            integration_status: "Not Tested".to_string(),
        }
    }

    fn from_test_data() -> Self {
        // Simulate ZKperf test data from our test results
        ZkperfMetrics {
            coverage_score: 10.0, // From test results
            performance_score: 52.5, // From test results
            test_success_rate: 50.0, // From test results
            test_duration_ms: 54324, // From test results
            files_generated: 3, // From test results
            test_categories_passed: 7, // From test results
            test_categories_failed: 7, // From test results
            coverage_analysis_status: "Poor".to_string(),
            performance_analysis_status: "Poor".to_string(),
            integration_status: "Poor".to_string(),
        }
    }

    fn get_coverage_color(&self) -> String {
        match self.coverage_score {
            score if score >= 80.0 => "#2ECC71".to_string(), // Green - Excellent
            score if score >= 60.0 => "#F39C12".to_string(), // Orange - Good
            score if score >= 40.0 => "#E67E22".to_string(), // Dark Orange - Fair
            score if score >= 20.0 => "#E74C3C".to_string(), // Red - Poor
            _ => "#C0392B".to_string(), // Dark Red - Very Poor
        }
    }

    fn get_performance_color(&self) -> String {
        match self.performance_score {
            score if score >= 80.0 => "#3498DB".to_string(), // Blue - Excellent
            score if score >= 60.0 => "#9B59B6".to_string(), // Purple - Good
            score if score >= 40.0 => "#E74C3C".to_string(), // Red - Fair
            score if score >= 20.0 => "#F39C12".to_string(), // Orange - Poor
            _ => "#D35400".to_string(), // Dark Orange - Very Poor
        }
    }

    fn get_test_status_color(&self) -> String {
        match self.test_success_rate {
            score if score >= 80.0 => "#27AE60".to_string(), // Green - Excellent
            score if score >= 60.0 => "#F39C12".to_string(), // Orange - Good
            score if score >= 40.0 => "#E67E22".to_string(), // Dark Orange - Fair
            score if score >= 20.0 => "#E74C3C".to_string(), // Red - Poor
            _ => "#C0392B".to_string(), // Dark Red - Very Poor
        }
    }

    fn get_coverage_badge(&self) -> String {
        format!("📊 {:.1}%", self.coverage_score)
    }

    fn get_performance_badge(&self) -> String {
        format!("⚡ {:.1}%", self.performance_score)
    }

    fn get_test_status_badge(&self) -> String {
        format!("🧪 {:.1}%", self.test_success_rate)
    }
}

#[derive(Debug, Clone)]
struct Repository {
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
    zkperf_metrics: ZkperfMetrics,
}

impl Repository {
    fn new(name: String, path: String) -> Self {
        Repository {
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
            zkperf_metrics: ZkperfMetrics::new(),
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

    fn integrate_zkperf_metrics(&mut self, zkperf_data: &ZkperfMetrics) {
        self.zkperf_metrics = zkperf_data.clone();
        
        // Adjust complexity based on ZKperf performance
        let zkperf_complexity_bonus = (zkperf_data.performance_score / 100.0) * 0.5;
        self.complexity_score += zkperf_complexity_bonus;
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
        let zkperf_size_modifier = (self.zkperf_metrics.coverage_score / 100.0 + self.zkperf_metrics.performance_score / 100.0) / 2.0;
        50.0 + (log_order * 15.0).min(150.0) * (1.0 + zkperf_size_modifier * 0.3)
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

    fn get_zkperf_tile_overlay(&self) -> String {
        let mut overlay = String::new();
        
        // Coverage badge
        overlay.push_str(&format!("<div style='position: absolute; top: 5px; left: 5px; background-color: {}; color: white; padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: bold;'>{}</div>", 
            self.zkperf_metrics.get_coverage_color(), 
            self.zkperf_metrics.get_coverage_badge()));
        
        // Performance badge
        overlay.push_str(&format!("<div style='position: absolute; top: 5px; right: 5px; background-color: {}; color: white; padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: bold;'>{}</div>", 
            self.zkperf_metrics.get_performance_color(), 
            self.zkperf_metrics.get_performance_badge()));
        
        // Test status badge
        overlay.push_str(&format!("<div style='position: absolute; bottom: 5px; left: 5px; background-color: {}; color: white; padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: bold;'>{}</div>", 
            self.zkperf_metrics.get_test_status_color(), 
            self.zkperf_metrics.get_test_status_badge()));
        
        overlay
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

    fn get_zkperf_description(&self) -> String {
        format!("ZKperf Coverage: {:.1}% | Performance: {:.1}% | Test Success: {:.1}%", 
            self.zkperf_metrics.coverage_score, 
            self.zkperf_metrics.performance_score, 
            self.zkperf_metrics.test_success_rate)
    }

    fn get_html_tile(&self) -> String {
        format!(
            "<div style='position: relative; width: {:.0}px; height: {:.0}px; background-color: {}; border: 2px solid #333; border-radius: 8px; margin: 10px; padding: 10px; font-family: Arial, sans-serif; box-shadow: 0 4px 8px rgba(0,0,0,0.1);'>
                <h3 style='margin: 0 0 5px 0; font-size: 14px; color: #333;'>{}</h3>
                <p style='margin: 0 0 5px 0; font-size: 11px; color: #666;'>{}</p>
                <p style='margin: 0 0 5px 0; font-size: 11px; color: #666;'>{}</p>
                <div style='margin-top: 10px; font-size: 10px; color: #888;'>
                    <div>🔢 Order: {}</div>
                    <div>📊 Complexity: {:.2}</div>
                    <div>🧪 ZKperf: {:.1}%</div>
                </div>
                {}
            </div>",
            self.get_tile_size(),
            self.get_tile_size(),
            self.get_tile_color(),
            self.name,
            self.get_group_theory_description(),
            self.get_zkperf_description(),
            self.group_order,
            self.complexity_score,
            (self.zkperf_metrics.coverage_score + self.zkperf_metrics.performance_score) / 2.0,
            self.get_zkperf_tile_overlay()
        )
    }
}

#[derive(Debug, Clone)]
struct RepositoryView {
    name: String,
    description: String,
    repositories: Vec<Repository>,
}

impl RepositoryView {
    fn new(name: String, description: String) -> Self {
        RepositoryView {
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

    fn get_zkperf_summary(&self) -> (f64, f64, f64) {
        if self.repositories.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let coverage_scores: Vec<f64> = self.repositories.iter().map(|r| r.zkperf_metrics.coverage_score).collect();
        let performance_scores: Vec<f64> = self.repositories.iter().map(|r| r.zkperf_metrics.performance_score).collect();
        let test_success_rates: Vec<f64> = self.repositories.iter().map(|r| r.zkperf_metrics.test_success_rate).collect();
        
        let avg_coverage = coverage_scores.iter().sum::<f64>() / coverage_scores.len() as f64;
        let avg_performance = performance_scores.iter().sum::<f64>() / performance_scores.len() as f64;
        let avg_test_success = test_success_rates.iter().sum::<f64>() / test_success_rates.len() as f64;
        
        (avg_coverage, avg_performance, avg_test_success)
    }

    fn generate_atlas_data(&self) -> String {
        let mut data = String::new();
        
        data.push_str(&format!("# Repository Mathematical Atlas with ZKperf Integration: {}\n\n", self.name));
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

        let (avg_cov, avg_perf, avg_test) = self.get_zkperf_summary();
        data.push_str("## ZKperf Integration Summary\n\n");
        data.push_str(&format!("- **Average Coverage**: {:.1}%\n", avg_cov));
        data.push_str(&format!("- **Average Performance**: {:.1}%\n", avg_perf));
        data.push_str(&format!("- **Average Test Success**: {:.1}%\n", avg_test));
        data.push_str("\n");

        data.push_str("## Repository Details with ZKperf Metrics\n\n");
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
            data.push_str(&format!("- **ZKperf Coverage**: {:.1}%\n", repo.zkperf_metrics.coverage_score));
            data.push_str(&format!("- **ZKperf Performance**: {:.1}%\n", repo.zkperf_metrics.performance_score));
            data.push_str(&format!("- **ZKperf Test Success**: {:.1}%\n", repo.zkperf_metrics.test_success_rate));
            data.push_str(&format!("- **ZKperf Files Generated**: {}\n", repo.zkperf_metrics.files_generated));
            data.push_str(&format!("- **ZKperf Test Duration**: {}ms\n", repo.zkperf_metrics.test_duration_ms));
            data.push_str(&format!("- **ZKperf Coverage Status**: {}\n", repo.zkperf_metrics.coverage_analysis_status));
            data.push_str(&format!("- **ZKperf Performance Status**: {}\n", repo.zkperf_metrics.performance_analysis_status));
            data.push_str(&format!("- **ZKperf Integration Status**: {}\n", repo.zkperf_metrics.integration_status));
            data.push_str("\n");
        }

        data
    }

    fn generate_html_visualization(&self) -> String {
        let mut html = String::new();
        
        html.push_str(&format!(
            "<!DOCTYPE html>
<html lang='en'>
<head>
    <meta charset='UTF-8'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0'>
    <title>Repository Mathematical Atlas with ZKperf - {}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
            background-color: #2c3e50;
            color: white;
            padding: 20px;
            border-radius: 10px;
        }}
        .summary {{
            display: flex;
            justify-content: space-around;
            margin-bottom: 30px;
            background-color: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .metric {{
            text-align: center;
            padding: 10px;
        }}
        .metric-value {{
            font-size: 24px;
            font-weight: bold;
            color: #2c3e50;
        }}
        .metric-label {{
            font-size: 14px;
            color: #7f8c8d;
        }}
        .tiles-container {{
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            gap: 20px;
            margin-bottom: 30px;
        }}
        .legend {{
            background-color: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .legend-item {{
            display: inline-block;
            margin: 5px;
            padding: 5px 10px;
            border-radius: 5px;
            color: white;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class='header'>
        <h1>🎯 Repository Mathematical Atlas with ZKperf Integration</h1>
        <h2>{}</h2>
        <p>{}</p>
    </div>
    
    <div class='summary'>
        <div class='metric'>
            <div class='metric-value'>{:.1}%</div>
            <div class='metric-label'>Avg Coverage</div>
        </div>
        <div class='metric'>
            <div class='metric-value'>{:.1}%</div>
            <div class='metric-label'>Avg Performance</div>
        </div>
        <div class='metric'>
            <div class='metric-value'>{:.1}%</div>
            <div class='metric-label'>Test Success</div>
        </div>
        <div class='metric'>
            <div class='metric-value'>{}</div>
            <div class='metric-label'>Repositories</div>
        </div>
    </div>
    
    <div class='tiles-container'>",
            self.name,
            self.name,
            self.description,
            self.get_zkperf_summary().0,
            self.get_zkperf_summary().1,
            self.get_zkperf_summary().2,
            self.repositories.len()
        ));

        for repo in &self.repositories {
            html.push_str(&repo.get_html_tile());
        }

        html.push_str(
            "</div>
    
    <div class='legend'>
        <h3>📊 Legend</h3>
        <div class='legend-item' style='background-color: #FF6B6B;'>Cyclic Groups</div>
        <div class='legend-item' style='background-color: #4ECDC4;'>Alternating Groups</div>
        <div class='legend-item' style='background-color: #45B7D1;'>Lie Type Groups</div>
        <div class='legend-item' style='background-color: #96CEB4;'>Sporadic Groups</div>
        <br>
        <div class='legend-item' style='background-color: #2ECC71;'>📊 Excellent Coverage</div>
        <div class='legend-item' style='background-color: #F39C12;'>📊 Good Coverage</div>
        <div class='legend-item' style='background-color: #E74C3C;'>📊 Poor Coverage</div>
        <div class='legend-item' style='background-color: #3498DB;'>⚡ Excellent Performance</div>
        <div class='legend-item' style='background-color: #E74C3C;'>⚡ Poor Performance</div>
        <div class='legend-item' style='background-color: #27AE60;'>🧷 Excellent Tests</div>
        <div class='legend-item' style='background-color: #E74C3C;'>🧷 Poor Tests</div>
    </div>
    
    <script>
        // Add interactive features
        document.addEventListener('DOMContentLoaded', function() {
            const tiles = document.querySelectorAll('.tiles-container > div');
            tiles.forEach(tile => {
                tile.addEventListener('click', function() {
                    const title = this.querySelector('h3').textContent;
                    const description = this.querySelector('p').textContent;
                    alert(`Repository: ${title}\\n\\n${description}`);
                });
                tile.style.cursor = 'pointer';
            });
        });
    </script>
</body>
</html>"
        );

        html
    }
}

struct RepositoryAtlasComposer {
    views: HashMap<String, RepositoryView>,
    zkperf_data: ZkperfMetrics,
}

impl RepositoryAtlasComposer {
    fn new() -> Self {
        RepositoryAtlasComposer {
            views: HashMap::new(),
            zkperf_data: ZkperfMetrics::from_test_data(),
        }
    }

    fn create_sample_repositories(&mut self) {
        let mut repos = vec![
            Repository::new("rust-lang/rust".to_string(), "/rust/rust".to_string()),
            Repository::new("torvalds/linux".to_string(), "/linux/linux".to_string()),
            Repository::new("microsoft/vscode".to_string(), "/vscode/vscode".to_string()),
            Repository::new("facebook/react".to_string(), "/react/react".to_string()),
            Repository::new("tensorflow/tensorflow".to_string(), "/tensorflow/tensorflow".to_string()),
        ];

        repos[0].stars = 75_000; repos[0].forks = 12_000; repos[0].contributors = 1_200; repos[0].language = "Rust".to_string(); repos[0].size_mb = 500; repos[0].last_commit_days = 1;
        repos[1].stars = 150_000; repos[1].forks = 50_000; repos[1].contributors = 2_500; repos[1].language = "C".to_string(); repos[1].size_mb = 1000; repos[1].last_commit_days = 0;
        repos[2].stars = 150_000; repos[2].forks = 25_000; repos[2].contributors = 1_800; repos[2].language = "TypeScript".to_string(); repos[2].size_mb = 300; repos[2].last_commit_days = 1;
        repos[3].stars = 200_000; repos[3].forks = 40_000; repos[3].contributors = 2_200; repos[3].language = "JavaScript".to_string(); repos[3].size_mb = 200; repos[3].last_commit_days = 0;
        repos[4].stars = 170_000; repos[4].forks = 70_000; repos[4].contributors = 5_000; repos[4].language = "Python".to_string(); repos[4].size_mb = 800; repos[4].last_commit_days = 2;

        for repo in &mut repos {
            repo.calculate_complexity();
            repo.classify_group_family();
            repo.integrate_zkperf_metrics(&self.zkperf_data);
        }

        let mut all_repos_view = RepositoryView::new("All Repositories".to_string(), "Complete mathematical analysis of all repositories with ZKperf integration".to_string());
        all_repos_view.repositories = repos;
        self.views.insert("all_repos".to_string(), all_repos_view);
    }

    fn create_view(&mut self, name: String, description: String) -> &mut RepositoryView {
        let view = RepositoryView::new(name.clone(), description);
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
            
            let (avg_cov, avg_perf, avg_test) = view.get_zkperf_summary();
            let mut zkperf_summary = HashMap::new();
            zkperf_summary.insert("average_coverage".to_string(), avg_cov.to_string());
            zkperf_summary.insert("average_performance".to_string(), avg_perf.to_string());
            zkperf_summary.insert("average_test_success".to_string(), avg_test.to_string());
            composition.insert("zkperf_summary".to_string(), format!("{:?}", zkperf_summary));
            
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
                
                // ZKperf metrics
                let mut zkperf_data = HashMap::new();
                zkperf_data.insert("coverage_score".to_string(), repo.zkperf_metrics.coverage_score.to_string());
                zkperf_data.insert("performance_score".to_string(), repo.zkperf_metrics.performance_score.to_string());
                zkperf_data.insert("test_success_rate".to_string(), repo.zkperf_metrics.test_success_rate.to_string());
                zkperf_data.insert("test_duration_ms".to_string(), repo.zkperf_metrics.test_duration_ms.to_string());
                zkperf_data.insert("files_generated".to_string(), repo.zkperf_metrics.files_generated.to_string());
                zkperf_data.insert("coverage_analysis_status".to_string(), repo.zkperf_metrics.coverage_analysis_status.clone());
                zkperf_data.insert("performance_analysis_status".to_string(), repo.zkperf_metrics.performance_analysis_status.clone());
                zkperf_data.insert("integration_status".to_string(), repo.zkperf_metrics.integration_status.clone());
                
                repo_data.insert("zkperf_metrics".to_string(), format!("{:?}", zkperf_data));
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
    println!("🎯 Repository Mathematical Atlas with ZKperf Integration - Analyzing Git Repositories Through Group Theory\n");
    
    let mut composer = RepositoryAtlasComposer::new();
    
    println!("📊 Creating sample repositories with mathematical classification and ZKperf integration...");
    composer.create_sample_repositories();
    
    println!("🔍 Creating specialized mathematical views with ZKperf metrics...");
    
    let cyclic_view = composer.create_view(
        "Cyclic Repositories".to_string(),
        "Repositories classified as Cyclic group family with ZKperf integration".to_string()
    );
    cyclic_view.repositories.retain(|r| r.group_family == "Cyclic");
    
    let high_complexity_view = composer.create_view(
        "High Complexity Repositories".to_string(),
        "Repositories with complexity score > 1.5 with ZKperf integration".to_string()
    );
    high_complexity_view.repositories.retain(|r| r.complexity_score > 1.5);
    
    let js_view = composer.create_view(
        "JavaScript Repositories".to_string(),
        "JavaScript language repositories with ZKperf integration".to_string()
    );
    js_view.repositories.retain(|r| r.language == "JavaScript");
    
    println!("\n📋 Available Views:");
    for view_name in composer.views.keys() {
        if let Some(view) = composer.views.get(view_name) {
            let (avg_cov, avg_perf, avg_test) = view.get_zkperf_summary();
            println!("  - {}: {} ({} repositories) | ZKperf: {:.1}% coverage, {:.1}% performance, {:.1}% test success", 
                view.name, view.description, view.repositories.len(), avg_cov, avg_perf, avg_test);
        }
    }
    
    println!("\n📤 Generating mathematical compositions with ZKperf data...");
    
    let views_to_export = vec!["all_repos", "Cyclic Repositories", "High Complexity Repositories", "JavaScript Repositories"];
    
    for view_name in views_to_export {
        match composer.generate_composition_json(view_name.to_string()) {
            Ok(json_content) => {
                let output_path = format!("./repository_atlas_output/simple_composition_{}.json", view_name.replace(" ", "_").to_lowercase());
                fs::write(&output_path, json_content).unwrap();
                println!("✅ Generated: {}", output_path);
            }
            Err(e) => println!("❌ Failed to generate {}: {}", view_name, e),
        }
    }
    
    println!("\n📚 Exporting complete atlas data with ZKperf integration...");
    if let Some(all_repos) = composer.views.get("all_repos") {
        let atlas_content = all_repos.generate_atlas_data();
        fs::write("./repository_atlas_output/simple_complete_atlas_with_zkperf.md", atlas_content).unwrap();
        println!("✅ Complete atlas with ZKperf exported to: ./repository_atlas_output/simple_complete_atlas_with_zkperf.md");
        
        let html_content = all_repos.generate_html_visualization();
        fs::write("./repository_atlas_output/simple_atlas_with_zkperf.html", html_content).unwrap();
        println!("✅ Interactive HTML visualization with ZKperf exported to: ./repository_atlas_output/simple_atlas_with_zkperf.html");
    }
    
    println!("\n🎉 Repository Mathematical Atlas with ZKperf Integration Analysis Complete!");
    println!("📁 Check the ./repository_atlas_output/ directory for generated files.");
    println!("🔬 Each tile now shows ZKperf coverage, performance, and test success metrics!");
}