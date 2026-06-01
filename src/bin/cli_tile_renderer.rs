use std::collections::HashMap;
use std::io::{self, Write};
use termion::{color, style, cursor};
use termion::raw::IntoRawMode;

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

    fn get_coverage_color(&self) -> String {
        match self.coverage_score {
            score if score >= 80.0 => color::Fg(color::Green).to_string(),
            score if score >= 60.0 => color::Fg(color::Yellow).to_string(),
            score if score >= 40.0 => color::Fg(color::LightYellow).to_string(),
            score if score >= 20.0 => color::Fg(color::LightRed).to_string(),
            _ => color::Fg(color::Red).to_string(),
        }
    }

    fn get_performance_color(&self) -> String {
        match self.performance_score {
            score if score >= 80.0 => color::Fg(color::Blue).to_string(),
            score if score >= 60.0 => color::Fg(color::Magenta).to_string(),
            score if score >= 40.0 => color::Fg(color::LightRed).to_string(),
            score if score >= 20.0 => color::Fg(color::Yellow).to_string(),
            _ => color::Fg(color::LightYellow).to_string(),
        }
    }

    fn get_test_status_color(&self) -> String {
        match self.test_success_rate {
            score if score >= 80.0 => color::Fg(color::Green).to_string(),
            score if score >= 60.0 => color::Fg(color::Yellow).to_string(),
            score if score >= 40.0 => color::Fg(color::LightYellow).to_string(),
            score if score >= 20.0 => color::Fg(color::LightRed).to_string(),
            _ => color::Fg(color::Red).to_string(),
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

    fn get_tile_color(&self) -> String {
        match self.group_family.as_str() {
            "Cyclic" => color::Fg(color::LightRed).to_string(),
            "Alternating" => color::Fg(color::Cyan).to_string(),
            "Lie Type" => color::Fg(color::LightBlue).to_string(),
            "Sporadic" => color::Fg(color::Green).to_string(),
            _ => color::Fg(color::Magenta).to_string(),
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

    fn render_tile<W: Write>(&self, mut writer: W, _selected: bool) -> io::Result<()> {
        let width = self.get_tile_width();
        let tile_color = self.get_tile_color();
        
        // Clear the tile area
        write!(writer, "{}", cursor::Goto(1, 1))?;
        
        // Draw top border
        write!(writer, "{}{}", style::Bold, color::Fg(color::White))?;
        for _ in 0..width {
            write!(writer, "─")?;
        }
        writeln!(writer, "{}{}", style::Reset, color::Fg(color::Reset))?;
        
        // Draw title and badges
        write!(writer, "{}│", cursor::Goto(1, 2))?;
        write!(writer, "{}{}{}{} ", tile_color, style::Bold, self.name, style::Reset)?;
        
        // Coverage badge (top left)
        write!(writer, "{}{}{} ", self.zkperf_metrics.get_coverage_color(), self.zkperf_metrics.get_coverage_badge(), color::Fg(color::Reset))?;
        
        // Performance badge (top right)
        write!(writer, "{}{}{} ", self.zkperf_metrics.get_performance_color(), self.zkperf_metrics.get_performance_badge(), color::Fg(color::Reset))?;
        
        writeln!(writer, "│")?;
        
        // Draw description line
        write!(writer, "{}│", cursor::Goto(1, 3))?;
        write!(writer, "{}{}{} ", tile_color, self.get_group_theory_description(), style::Reset)?;
        for _ in 0..(width - 2 - self.get_group_theory_description().len()) {
            write!(writer, " ")?;
        }
        writeln!(writer, "│")?;
        
        // Draw complexity and ZKperf line
        write!(writer, "{}│", cursor::Goto(1, 4))?;
        write!(writer, "🔢 Order: {} | 📊 Complexity: {:.2} | 🧪 ZKperf: {:.1}%", 
            self.group_order, self.complexity_score, 
            (self.zkperf_metrics.coverage_score + self.zkperf_metrics.performance_score) / 2.0)?;
        for _ in 0..(width - 2 - 60) {
            write!(writer, " ")?;
        }
        writeln!(writer, "│")?;
        
        // Draw test status badge (bottom left)
        write!(writer, "{}│", cursor::Goto(1, 5))?;
        write!(writer, "{}{}{} ", self.zkperf_metrics.get_test_status_color(), self.zkperf_metrics.get_test_status_badge(), color::Fg(color::Reset))?;
        
        // Draw bottom border
        write!(writer, "{}{}", style::Bold, color::Fg(color::White))?;
        for _ in 0..width {
            write!(writer, "─")?;
        }
        writeln!(writer, "{}{}", style::Reset, color::Fg(color::Reset))?;
        
        Ok(())
    }

    fn render_detailed<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writeln!(writer, "{}🎯 Repository: {}{}", style::Bold, self.name, style::Reset)?;
        writeln!(writer, "   Path: {}", self.path)?;
        writeln!(writer, "   Group Family: {}", self.group_family)?;
        writeln!(writer, "   Group Order: {}", self.group_order)?;
        writeln!(writer, "   Group Rank: {}", self.group_rank)?;
        writeln!(writer, "   Complexity Score: {:.2}", self.complexity_score)?;
        writeln!(writer, "   Simple Subgroups: {}", self.simple_subgroups.join(", "))?;
        writeln!(writer, "   Stars: {}", self.stars)?;
        writeln!(writer, "   Forks: {}", self.forks)?;
        writeln!(writer, "   Contributors: {}", self.contributors)?;
        writeln!(writer, "   Language: {}", self.language)?;
        writeln!(writer, "   Size: {} MB", self.size_mb)?;
        writeln!(writer, "   Last Commit: {} days ago", self.last_commit_days)?;
        writeln!(writer, "   Tile Width: {}px", self.get_tile_width())?;
        writeln!(writer, "   Tile Color: {}", self.get_tile_color())?;
        writeln!(writer, "   Group Theory Description: {}", self.get_group_theory_description())?;
        writeln!(writer, "   ZKperf Coverage: {:.1}%", self.zkperf_metrics.coverage_score)?;
        writeln!(writer, "   ZKperf Performance: {:.1}%", self.zkperf_metrics.performance_score)?;
        writeln!(writer, "   ZKperf Test Success: {:.1}%", self.zkperf_metrics.test_success_rate)?;
        writeln!(writer, "   ZKperf Files Generated: {}", self.zkperf_metrics.files_generated)?;
        writeln!(writer, "   ZKperf Test Duration: {}ms", self.zkperf_metrics.test_duration_ms)?;
        writeln!(writer, "   ZKperf Coverage Status: {}", self.zkperf_metrics.coverage_analysis_status)?;
        writeln!(writer, "   ZKperf Performance Status: {}", self.zkperf_metrics.performance_analysis_status)?;
        writeln!(writer, "   ZKperf Integration Status: {}", self.zkperf_metrics.integration_status)?;
        writeln!(writer)?;
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

    fn render_summary<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writeln!(writer, "{}🎯 Repository Mathematical Atlas with ZKperf - {}{}", style::Bold, self.name, style::Reset)?;
        writeln!(writer, "{}Description: {}{}", style::Bold, self.description, style::Reset)?;
        writeln!(writer)?;
        
        let family_dist = self.get_family_distribution();
        writeln!(writer, "{}📊 Family Distribution:{}", style::Bold, style::Reset)?;
        for (family, count) in family_dist {
            writeln!(writer, "   {}: {} repositories", family, count)?;
        }
        writeln!(writer)?;
        
        let (min_comp, max_comp, avg_comp) = self.get_complexity_stats();
        writeln!(writer, "{}📈 Complexity Statistics:{}", style::Bold, style::Reset)?;
        writeln!(writer, "   Minimum Complexity: {:.2}", min_comp)?;
        writeln!(writer, "   Maximum Complexity: {:.2}", max_comp)?;
        writeln!(writer, "   Average Complexity: {:.2}", avg_comp)?;
        writeln!(writer)?;
        
        let (avg_cov, avg_perf, avg_test) = self.get_zkperf_summary();
        writeln!(writer, "{}🔬 ZKperf Integration Summary:{}", style::Bold, style::Reset)?;
        writeln!(writer, "   Average Coverage: {:.1}%", avg_cov)?;
        writeln!(writer, "   Average Performance: {:.1}%", avg_perf)?;
        writeln!(writer, "   Average Test Success: {:.1}%", avg_test)?;
        writeln!(writer)?;
        
        Ok(())
    }

    fn render_tiles<W: Write>(&self, _writer: W, selected_index: Option<usize>) -> io::Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let mut handle = stdout;
        
        // Clear screen
        write!(handle, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;
        
        // Render header
        self.render_summary(&mut handle)?;
        
        // Calculate tile layout
        let tile_width = 40;
        let tile_height = 6;
        let tiles_per_row = (80 / tile_width).max(1);
        
        // Render tiles
        for (i, repo) in self.repositories.iter().enumerate() {
            let row = i / tiles_per_row;
            let col = i % tiles_per_row;
            let x = col * tile_width + 1;
            let y = row * tile_height + 10; // Offset for header
            
            // Move cursor to tile position
            write!(handle, "{}", cursor::Goto(x as u16, y as u16))?;
            
            // Render tile
            repo.render_tile(&mut handle, Some(i) == selected_index)?;
        }
        
// Render legend
         write!(handle, "{}", cursor::Goto(1, ((self.repositories.len() / tiles_per_row + 1) * tile_height + 12) as u16))?;
         self.render_legend(&mut handle)?;
         
         // Render instructions
         write!(handle, "{}", cursor::Goto(1, ((self.repositories.len() / tiles_per_row + 1) * tile_height + 14) as u16))?;
        writeln!(handle, "{}Controls:{} Arrow keys to navigate, Enter for details, Q to quit", style::Bold, style::Reset)?;
        
        handle.flush()?;
        Ok(())
    }

    fn render_legend<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writeln!(writer, "{}📊 Legend:{}", style::Bold, style::Reset)?;
        writeln!(writer, "   🟢 Cyclic Groups - Prime order simple groups")?;
        writeln!(writer, "   🟢 Alternating Groups - Non-abelian simple groups")?;
        writeln!(writer, "   🔵 Lie Type Groups - Classical simple groups")?;
        writeln!(writer, "   🟢 Sporadic Groups - Exceptional simple groups")?;
        writeln!(writer)?;
        writeln!(writer, "{}🔬 ZKperf Indicators:{}", style::Bold, style::Reset)?;
        writeln!(writer, "   📊 Coverage: Green (≥80%) | Yellow (60-79%) | Red (<60%)")?;
        writeln!(writer, "   ⚡ Performance: Blue (≥80%) | Purple (60-79%) | Red (<60%)")?;
        writeln!(writer, "   🧪 Test Success: Green (≥80%) | Yellow (60-79%) | Red (<60%)")?;
        Ok(())
    }

    fn render_detailed_view<W: Write>(&self, writer: W, index: usize) -> io::Result<()> {
        if index < self.repositories.len() {
            self.repositories[index].render_detailed(writer)?;
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
            view.render_tiles(io::stdout(), Some(self.selected_index))?;
        }
        Ok(())
    }

    fn render_detailed_view(&mut self) -> io::Result<()> {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.render_detailed_view(io::stdout(), self.selected_index)?;
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
        let stdin = io::stdin();
        let mut buffer = String::new();
        
        stdin.read_line(&mut buffer)?;
        let input = buffer.trim();
        
        match input {
            "q" | "Q" => Ok(false), // Quit
            "enter" | "" => { // Show detailed view
                self.render_detailed_view()?;
                println!("Press Enter to return to tile view...");
                let _ = stdin.read_line(&mut buffer);
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
    
    // Set terminal to raw mode
    let stdout = io::stdout().into_raw_mode()?;
    let mut handle = stdout;
    
    // Clear screen and render initial view
    write!(handle, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;
    composer.render_current_view()?;
    
    // Main event loop
    loop {
        match composer.handle_input() {
            Ok(false) => break,
            Ok(true) => {
                write!(handle, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;
                composer.render_current_view()?;
            }
            Err(e) => {
                eprintln!("Error handling input: {}", e);
                break;
            }
        }
    }
    
    // Reset terminal
    write!(handle, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;
    println!("🎉 Thanks for using the Repository Mathematical Atlas CLI Tile Renderer!");
    
    Ok(())
}