use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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

struct RepositoryAtlasComposer {
    views: HashMap<String, RepositoryView>,
}

impl RepositoryAtlasComposer {
    fn new() -> Self {
        RepositoryAtlasComposer {
            views: HashMap::new(),
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
        }

        let mut all_repos_view = RepositoryView::new("All Repositories".to_string(), "Complete mathematical analysis of all repositories".to_string());
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
    println!("🎯 Repository Mathematical Atlas - Analyzing Git Repositories Through Group Theory\n");
    
    let mut composer = RepositoryAtlasComposer::new();
    
    println!("📊 Creating sample repositories with mathematical classification...");
    composer.create_sample_repositories();
    
    println!("🔍 Creating specialized mathematical views...");
    
    let cyclic_view = composer.create_view(
        "Cyclic Repositories".to_string(),
        "Repositories classified as Cyclic group family".to_string()
    );
    cyclic_view.repositories.retain(|r| r.group_family == "Cyclic");
    
    let high_complexity_view = composer.create_view(
        "High Complexity Repositories".to_string(),
        "Repositories with complexity score > 1.5".to_string()
    );
    high_complexity_view.repositories.retain(|r| r.complexity_score > 1.5);
    
    let js_view = composer.create_view(
        "JavaScript Repositories".to_string(),
        "JavaScript language repositories".to_string()
    );
    js_view.repositories.retain(|r| r.language == "JavaScript");
    
    println!("\n📋 Available Views:");
    for view_name in composer.views.keys() {
        if let Some(view) = composer.views.get(view_name) {
            println!("  - {}: {} ({} repositories)", view.name, view.description, view.repositories.len());
        }
    }
    
    println!("\n📤 Generating mathematical compositions...");
    
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
    
    println!("\n📚 Exporting complete atlas data...");
    if let Some(all_repos) = composer.views.get("all_repos") {
        let atlas_content = all_repos.generate_atlas_data();
        fs::write("./repository_atlas_output/simple_complete_atlas.md", atlas_content).unwrap();
        println!("✅ Complete atlas exported to: ./repository_atlas_output/simple_complete_atlas.md");
    }
    
    println!("\n🎉 Repository Mathematical Atlas Analysis Complete!");
    println!("📁 Check the ./repository_atlas_output/ directory for generated files.");
}