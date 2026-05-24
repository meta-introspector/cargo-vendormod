use std::collections::HashMap;
use std::io;

#[derive(Debug, Clone)]
struct ZkperfMetrics {
    coverage_score: f64,
    performance_score: f64,
    test_success_rate: f64,
    test_duration_ms: u64,
    files_generated: usize,
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
            coverage_analysis_status: "Not Tested".to_string(),
            performance_analysis_status: "Not Tested".to_string(),
            integration_status: "Not Tested".to_string(),
        }
    }

    fn from_test_data() -> Self {
        ZkperfMetrics {
            coverage_score: 10.0,
            performance_score: 52.5,
            test_success_rate: 50.0,
            test_duration_ms: 54324,
            files_generated: 3,
            coverage_analysis_status: "Poor".to_string(),
            performance_analysis_status: "Poor".to_string(),
            integration_status: "Poor".to_string(),
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

    fn get_coverage_color(&self) -> String {
        match self.coverage_score {
            score if score >= 80.0 => "[🟢]".to_string(),
            score if score >= 60.0 => "[🟠]".to_string(),
            score if score >= 40.0 => "[🟡]".to_string(),
            score if score >= 20.0 => "[🔴]".to_string(),
            _ => "[🔴]".to_string(),
        }
    }

    fn get_performance_color(&self) -> String {
        match self.performance_score {
            score if score >= 80.0 => "[🔵]".to_string(),
            score if score >= 60.0 => "[🟣]".to_string(),
            score if score >= 40.0 => "[🔴]".to_string(),
            score if score >= 20.0 => "[🟠]".to_string(),
            _ => "[🟡]".to_string(),
        }
    }

    fn get_test_status_color(&self) -> String {
        match self.test_success_rate {
            score if score >= 80.0 => "[🟢]".to_string(),
            score if score >= 60.0 => "[🟠]".to_string(),
            score if score >= 40.0 => "[🟡]".to_string(),
            score if score >= 20.0 => "[🔴]".to_string(),
            _ => "[🔴]".to_string(),
        }
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

    fn get_tile_width(&self) -> usize {
        let log_order = (self.group_order.parse::<usize>().unwrap_or(1) as f64 + 1.0).log10();
        let zkperf_size_modifier = (self.zkperf_metrics.coverage_score / 100.0 + self.zkperf_metrics.performance_score / 100.0) / 2.0;
        let base_width = (50.0 + (log_order * 15.0).min(150.0)) as usize;
        (base_width * (1 + (zkperf_size_modifier * 0.3) as usize)).min(80)
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

    fn render_tile(&self, selected: bool) -> io::Result<()> {
        let width = self.get_tile_width();
        let selected_marker = if selected { "► " } else { "  " };
        
        // Draw top border
        print!("{}", selected_marker);
        for _ in 0..width {
            print!("─");
        }
        println!();
        
        // Draw title and badges
        print!("{}│", selected_marker);
        print!("{}{} ", self.name, self.get_group_theory_description());
        
        // Coverage badge
        print!("{}{} ", self.zkperf_metrics.get_coverage_color(), self.zkperf_metrics.get_coverage_badge());
        
        // Performance badge
        print!("{}{} ", self.zkperf_metrics.get_performance_color(), self.zkperf_metrics.get_performance_badge());
        
        // Test status badge
        print!("{}{} ", self.zkperf_metrics.get_test_status_color(), self.zkperf_metrics.get_test_status_badge());
        
        for _ in 0..(width - 2 - self.name.len() - self.get_group_theory_description().len() - 30) {
            print!(" ");
        }
        println!("│");
        
        // Draw complexity and ZKperf line
        print!("{}│", selected_marker);
        print!("🔢 Order: {} | 📊 Complexity: {:.2} | 🧪 ZKperf: {:.1}%", 
            self.group_order, self.complexity_score, 
            (self.zkperf_metrics.coverage_score + self.zkperf_metrics.performance_score) / 2.0);
        for _ in 0..(width - 2 - 60) {
            print!(" ");
        }
        println!("│");
        
        // Draw bottom border
        print!("{}", selected_marker);
        for _ in 0..width {
            print!("─");
        }
        println!();
        
        Ok(())
    }

    fn render_detailed(&self) -> io::Result<()> {
        println!("🎯 Repository: {}", self.name);
        println!("   Path: {}", self.path);
        println!("   Group Family: {}", self.group_family);
        println!("   Group Order: {}", self.group_order);
        println!("   Group Rank: {}", self.group_rank);
        println!("   Complexity Score: {:.2}", self.complexity_score);
        println!("   Simple Subgroups: {}", self.simple_subgroups.join(", "));
        println!("   Stars: {}", self.stars);
        println!("   Forks: {}", self.forks);
        println!("   Contributors: {}", self.contributors);
        println!("   Language: {}", self.language);
        println!("   Size: {} MB", self.size_mb);
        println!("   Last Commit: {} days ago", self.last_commit_days);
        println!("   Tile Width: {}px", self.get_tile_width());
        println!("   Group Theory Description: {}", self.get_group_theory_description());
        println!("   ZKperf Coverage: {:.1}% {}", self.zkperf_metrics.coverage_score, self.zkperf_metrics.get_coverage_color());
        println!("   ZKperf Performance: {:.1}% {}", self.zkperf_metrics.performance_score, self.zkperf_metrics.get_performance_color());
        println!("   ZKperf Test Success: {:.1}% {}", self.zkperf_metrics.test_success_rate, self.zkperf_metrics.get_test_status_color());
        println!("   ZKperf Files Generated: {}", self.zkperf_metrics.files_generated);
        println!("   ZKperf Test Duration: {}ms", self.zkperf_metrics.test_duration_ms);
        println!("   ZKperf Coverage Status: {}", self.zkperf_metrics.coverage_analysis_status);
        println!("   ZKperf Performance Status: {}", self.zkperf_metrics.performance_analysis_status);
        println!("   ZKperf Integration Status: {}", self.zkperf_metrics.integration_status);
        println!();
        Ok(())
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

    fn render_summary(&self) -> io::Result<()> {
        println!("🎯 Repository Mathematical Atlas with ZKperf - {}", self.name);
        println!("Description: {}", self.description);
        println!();
        
        let family_dist = self.get_family_distribution();
        println!("📊 Family Distribution:");
        for (family, count) in family_dist {
            println!("   {}: {} repositories", family, count);
        }
        println!();
        
        let (min_comp, max_comp, avg_comp) = self.get_complexity_stats();
        println!("📈 Complexity Statistics:");
        println!("   Minimum Complexity: {:.2}", min_comp);
        println!("   Maximum Complexity: {:.2}", max_comp);
        println!("   Average Complexity: {:.2}", avg_comp);
        println!();
        
        let (avg_cov, avg_perf, avg_test) = self.get_zkperf_summary();
        println!("🔬 ZKperf Integration Summary:");
        println!("   Average Coverage: {:.1}% {}", avg_cov, self.repositories[0].zkperf_metrics.get_coverage_color());
        println!("   Average Performance: {:.1}% {}", avg_perf, self.repositories[0].zkperf_metrics.get_performance_color());
        println!("   Average Test Success: {:.1}% {}", avg_test, self.repositories[0].zkperf_metrics.get_test_status_color());
        println!();
        
        Ok(())
    }

    fn render_tiles(&self, selected_index: usize) -> io::Result<()> {
        // Clear screen
        print!("{}[2J", 27 as char); // Clear screen
        print!("{}[H", 27 as char); // Move cursor to home
        
        // Render header
        self.render_summary()?;
        
        // Calculate tile layout
        let tile_width = 40;
        let tile_height = 4;
        let tiles_per_row = 2;
        
        // Render tiles
        for (i, repo) in self.repositories.iter().enumerate() {
            let row = i / tiles_per_row;
            let col = i % tiles_per_row;
            let y = row * tile_height + 8; // Offset for header
            
            // Move cursor to tile position
            print!("{}[{};{}H", 27 as char, y + 1, col * tile_width + 1);
            
            // Render tile
            repo.render_tile(i == selected_index)?;
        }
        
        // Render legend
        print!("{}[{};1H", 27 as char, self.repositories.len() / tiles_per_row + 1 + tile_height + 2);
        self.render_legend()?;
        
        // Render instructions
        print!("{}[{};1H", 27 as char, self.repositories.len() / tiles_per_row + 1 + tile_height + 4);
        println!("Controls: ↑↓ Arrow keys to navigate, Enter for details, Q to quit");
        
        Ok(())
    }

    fn render_legend(&self) -> io::Result<()> {
        println!("📊 Legend:");
        println!("   🟢 Cyclic Groups - Prime order simple groups");
        println!("   🟢 Alternating Groups - Non-abelian simple groups");
        println!("   🔵 Lie Type Groups - Classical simple groups");
        println!("   🟢 Sporadic Groups - Exceptional simple groups");
        println!();
        println!("🔬 ZKperf Indicators:");
        println!("   📊 Coverage: Green (≥80%) | Yellow (60-79%) | Red (<60%)");
        println!("   ⚡ Performance: Blue (≥80%) | Purple (60-79%) | Red (<60%)");
        println!("   🧪 Test Success: Green (≥80%) | Yellow (60-79%) | Red (<60%)");
        Ok(())
    }

    fn render_detailed_view(&self, index: usize) -> io::Result<()> {
        if index < self.repositories.len() {
            self.repositories[index].render_detailed()?;
        }
        Ok(())
    }
}

struct RepositoryAtlasComposer {
    views: HashMap<String, RepositoryView>,
    zkperf_data: ZkperfMetrics,
    current_view: String,
    selected_index: usize,
}

impl RepositoryAtlasComposer {
    fn new() -> Self {
        RepositoryAtlasComposer {
            views: HashMap::new(),
            zkperf_data: ZkperfMetrics::from_test_data(),
            current_view: "all_repos".to_string(),
            selected_index: 0,
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

    fn render_current_view(&mut self) -> io::Result<()> {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.render_tiles(self.selected_index)?;
        }
        Ok(())
    }

    fn render_detailed_view(&mut self) -> io::Result<()> {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.render_detailed_view(self.selected_index)?;
        }
        Ok(())
    }

    fn move_selection(&mut self, direction: i32) {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            let new_index = (self.selected_index as i32 + direction).max(0).min(view.repositories.len() as i32 - 1) as usize;
            self.selected_index = new_index;
        }
    }

    fn handle_input(&mut self) -> io::Result<bool> {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let input = buffer.trim();
        
        match input {
            "q" | "Q" => Ok(false), // Quit
            "" => { // Show detailed view
                self.render_detailed_view()?;
                println!("Press Enter to return to tile view...");
                let _ = io::stdin().read_line(&mut buffer);
                Ok(true)
            }
            "up" | "k" => {
                self.move_selection(-1);
                Ok(true)
            }
            "down" | "j" => {
                self.move_selection(1);
                Ok(true)
            }
            "left" | "h" => {
                self.move_selection(-1);
                Ok(true)
            }
            "right" | "l" => {
                self.move_selection(1);
                Ok(true)
            }
            _ => Ok(true),
        }
    }
}

fn main() -> io::Result<()> {
    println!("🎯 Repository Mathematical Atlas with ZKperf - CLI Tile Renderer");
    println!("================================================================");
    println!("Initializing...");
    
    let mut composer = RepositoryAtlasComposer::new();
    composer.create_sample_repositories();
    
    println!("✅ Created sample repositories with mathematical classification and ZKperf integration");
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
    
    println!("\n🎮 Starting interactive tile renderer...");
    println!("Controls: Arrow keys to navigate, Enter for details, Q to quit");
    
    // Main event loop
    loop {
        composer.render_current_view()?;
        
        match composer.handle_input() {
            Ok(false) => break,
            Ok(true) => {
                // Continue loop
            }
            Err(e) => {
                eprintln!("Error handling input: {}", e);
                break;
            }
        }
    }
    
    // Clear screen and show goodbye message
    print!("{}[2J", 27 as char);
    print!("{}[H", 27 as char);
    println!("🎉 Thanks for using the Repository Mathematical Atlas CLI Tile Renderer!");
    
    Ok(())
}