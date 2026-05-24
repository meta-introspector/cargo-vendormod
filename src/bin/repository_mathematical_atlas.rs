use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Repository structure for mathematical classification
#[derive(Debug, Clone)]
struct Repository {
    name: String,
    path: String,
    stars: usize,
    forks: usize,
    contributors: usize,
    is_fork: bool,
    has_issues: bool,
    has_wiki: bool,
    language: String,
    size_mb: usize,
    last_commit_days: usize,
    complexity_score: f64,
    group_family: String,
    mathematical_properties: HashMap<String, String>,
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
            has_issues: false,
            has_wiki: false,
            language: "Unknown".to_string(),
            size_mb: 0,
            last_commit_days: 0,
            complexity_score: 0.0,
            group_family: "Unclassified".to_string(),
            mathematical_properties: HashMap::new(),
        }
    }

    fn calculate_complexity(&mut self) {
        // Complexity based on multiple factors
        let mut complexity = 0.0;
        
        // Star complexity (logarithmic scale)
        complexity += (self.stars as f64 + 1.0).log10() * 0.3;
        
        // Fork complexity
        complexity += (self.forks as f64 + 1.0).log10() * 0.2;
        
        // Contributor complexity
        complexity += (self.contributors as f64 + 1.0).log10() * 0.25;
        
        // Size complexity
        complexity += (self.size_mb as f64 + 1.0).log10() * 0.15;
        
        // Activity complexity (inverse of last_commit_days)
        if self.last_commit_days > 0 {
            complexity += (365.0 / self.last_commit_days as f64).log10() * 0.1;
        }
        
        // Feature complexity
        if self.has_issues { complexity += 0.05; }
        if self.has_wiki { complexity += 0.05; }
        
        self.complexity_score = complexity;
    }

    fn classify_group_family(&mut self) {
        // Classify repositories into mathematical group families
        
        if self.is_fork {
            self.group_family = "Cyclic".to_string();
            self.mathematical_properties.insert("order".to_string(), "2".to_string());
            self.mathematical_properties.insert("type".to_string(), "fork".to_string());
        } else if self.complexity_score < 1.0 {
            self.group_family = "Cyclic".to_string();
            self.mathematical_properties.insert("order".to_string(), "prime".to_string());
            self.mathematical_properties.insert("type".to_string(), "simple".to_string());
        } else if self.complexity_score < 2.0 {
            self.group_family = "Alternating".to_string();
            self.mathematical_properties.insert("order".to_string(), "composite".to_string());
            self.mathematical_properties.insert("type".to_string(), "alternating".to_string());
        } else if self.complexity_score < 3.0 {
            self.group_family = "Lie Type".to_string();
            self.mathematical_properties.insert("order".to_string(), "large".to_string());
            self.mathematical_properties.insert("type".to_string(), "lie".to_string());
        } else if self.complexity_score < 4.0 {
            self.group_family = "Sporadic".to_string();
            self.mathematical_properties.insert("order".to_string(), "exceptional".to_string());
            self.mathematical_properties.insert("type".to_string(), "sporadic".to_string());
        } else {
            self.group_family = "Twisted Lie".to_string();
            self.mathematical_properties.insert("order".to_string(), "monster".to_string());
            self.mathematical_properties.insert("type".to_string(), "twisted".to_string());
        }
    }

    fn get_tile_size(&self) -> f64 {
        // Tile size based on complexity score (logarithmic scale)
        50.0 + (self.complexity_score * 20.0).min(150.0)
    }

    fn get_tile_color(&self) -> String {
        match self.group_family.as_str() {
            "Cyclic" => "#FF6B6B".to_string(),      // Red
            "Alternating" => "#4ECDC4".to_string(),   // Teal
            "Lie Type" => "#45B7D1".to_string(),      // Blue
            "Sporadic" => "#96CEB4".to_string(),      // Green
            "Twisted Lie" => "#FFEAA7".to_string(),   // Yellow
            _ => "#DDA0DD".to_string(),               // Plum
        }
    }
}

// Repository view for mathematical analysis
#[derive(Debug, Clone)]
struct RepositoryView {
    name: String,
    description: String,
    family_filter: Option<String>,
    complexity_range: Option<(f64, f64)>,
    language_filter: Option<String>,
    min_stars: Option<usize>,
    max_stars: Option<usize>,
    repositories: Vec<Repository>,
    shared_info: String,
}

impl RepositoryView {
    fn new(name: String, description: String) -> Self {
        RepositoryView {
            name,
            description,
            family_filter: None,
            complexity_range: None,
            language_filter: None,
            min_stars: None,
            max_stars: None,
            repositories: Vec::new(),
            shared_info: "Private view".to_string(),
        }
    }

    fn filter_by_family(&mut self, family: String) {
        self.family_filter = Some(family);
        self.apply_filters();
    }

    fn filter_by_complexity(&mut self, min: f64, max: f64) {
        self.complexity_range = Some((min, max));
        self.apply_filters();
    }

    fn filter_by_language(&mut self, language: String) {
        self.language_filter = Some(language);
        self.apply_filters();
    }

    fn filter_by_stars(&mut self, min: Option<usize>, max: Option<usize>) {
        self.min_stars = min;
        self.max_stars = max;
        self.apply_filters();
    }

    fn apply_filters(&mut self) {
        self.repositories.retain(|repo| {
            // Family filter
            if let Some(ref family) = self.family_filter {
                if repo.group_family != *family {
                    return false;
                }
            }

            // Complexity range filter
            if let Some((min, max)) = self.complexity_range {
                if repo.complexity_score < min || repo.complexity_score > max {
                    return false;
                }
            }

            // Language filter
            if let Some(ref language) = self.language_filter {
                if repo.language != *language {
                    return false;
                }
            }

            // Stars filter
            if let Some(min) = self.min_stars {
                if repo.stars < min {
                    return false;
                }
            }

            if let Some(max) = self.max_stars {
                if repo.stars > max {
                    return false;
                }
            }

            true
        });
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
        let max = complexities.iter().fold(0.0_f64, |a, &b| a.max(b));
        let avg = complexities.iter().sum::<f64>() / complexities.len() as f64;
        
        (min, max, avg)
    }

    fn generate_atlas_data(&self) -> String {
        let mut data = String::new();
        
        data.push_str(&format!("# Repository Mathematical Atlas: {}\n\n", self.name));
        data.push_str(&format!("**Description**: {}\n\n", self.description));
        
        // Family distribution
        let family_dist = self.get_family_distribution();
        data.push_str("## Family Distribution\n\n");
        for (family, count) in family_dist {
            data.push_str(&format!("- **{}**: {} repositories\n", family, count));
        }
        data.push_str("\n");

        // Complexity statistics
        let (min_comp, max_comp, avg_comp) = self.get_complexity_stats();
        data.push_str("## Complexity Statistics\n\n");
        data.push_str(&format!("- **Minimum Complexity**: {:.2}\n", min_comp));
        data.push_str(&format!("- **Maximum Complexity**: {:.2}\n", max_comp));
        data.push_str(&format!("- **Average Complexity**: {:.2}\n", avg_comp));
        data.push_str("\n");

        // Repository details
        data.push_str("## Repository Details\n\n");
        for (i, repo) in self.repositories.iter().enumerate() {
            data.push_str(&format!("### {}. {}\n", i + 1, repo.name));
            data.push_str(&format!("- **Path**: {}\n", repo.path));
            data.push_str(&format!("- **Group Family**: {}\n", repo.group_family));
            data.push_str(&format!("- **Complexity Score**: {:.2}\n", repo.complexity_score));
            data.push_str(&format!("- **Stars**: {}\n", repo.stars));
            data.push_str(&format!("- **Forks**: {}\n", repo.forks));
            data.push_str(&format!("- **Contributors**: {}\n", repo.contributors));
            data.push_str(&format!("- **Language**: {}\n", repo.language));
            data.push_str(&format!("- **Size**: {} MB\n", repo.size_mb));
            data.push_str(&format!("- **Last Commit**: {} days ago\n", repo.last_commit_days));
            data.push_str(&format!("- **Tile Size**: {:.0}px\n", repo.get_tile_size()));
            data.push_str(&format!("- **Tile Color**: {}\n", repo.get_tile_color()));
            data.push_str("\n");
        }

        data
    }
}

// Repository Atlas Composer
struct RepositoryAtlasComposer {
    views: HashMap<String, RepositoryView>,
    shared_views: HashMap<String, RepositoryView>,
}

impl RepositoryAtlasComposer {
    fn new() -> Self {
        RepositoryAtlasComposer {
            views: HashMap::new(),
            shared_views: HashMap::new(),
        }
    }

    fn create_sample_repositories(&mut self) {
        // Create sample repositories with mathematical properties
        let mut repos = vec![
            Repository::new("rust-lang/rust".to_string(), "/rust/rust".to_string()),
            Repository::new("torvalds/linux".to_string(), "/linux/linux".to_string()),
            Repository::new("microsoft/vscode".to_string(), "/vscode/vscode".to_string()),
            Repository::new("facebook/react".to_string(), "/react/react".to_string()),
            Repository::new("tensorflow/tensorflow".to_string(), "/tensorflow/tensorflow".to_string()),
            Repository::new("pytorch/pytorch".to_string(), "/pytorch/pytorch".to_string()),
            Repository::new("vuejs/vue".to_string(), "/vue/vue".to_string()),
            Repository::new("angular/angular".to_string(), "/angular/angular".to_string()),
            Repository::new("nodejs/node".to_string(), "/node/node".to_string()),
            Repository::new("docker/docker".to_string(), "/docker/docker".to_string()),
        ];

        // Set properties for each repository
        repos[0].stars = 75_000; repos[0].forks = 12_000; repos[0].contributors = 1_200; repos[0].language = "Rust".to_string(); repos[0].size_mb = 500; repos[0].last_commit_days = 1;
        repos[1].stars = 150_000; repos[1].forks = 50_000; repos[1].contributors = 2_500; repos[1].language = "C".to_string(); repos[1].size_mb = 1000; repos[1].last_commit_days = 0;
        repos[2].stars = 150_000; repos[2].forks = 25_000; repos[2].contributors = 1_800; repos[2].language = "TypeScript".to_string(); repos[2].size_mb = 300; repos[2].last_commit_days = 1;
        repos[3].stars = 200_000; repos[3].forks = 40_000; repos[3].contributors = 2_200; repos[3].language = "JavaScript".to_string(); repos[3].size_mb = 200; repos[3].last_commit_days = 0;
        repos[4].stars = 170_000; repos[4].forks = 70_000; repos[4].contributors = 5_000; repos[4].language = "Python".to_string(); repos[4].size_mb = 800; repos[4].last_commit_days = 2;
        repos[5].stars = 70_000; repos[5].forks = 15_000; repos[5].contributors = 1_500; repos[5].language = "Python".to_string(); repos[5].size_mb = 600; repos[5].last_commit_days = 1;
        repos[6].stars = 200_000; repos[6].forks = 35_000; repos[6].contributors = 1_900; repos[6].language = "JavaScript".to_string(); repos[6].size_mb = 150; repos[6].last_commit_days = 0;
        repos[7].stars = 90_000; repos[7].forks = 25_000; repos[7].contributors = 1_300; repos[7].language = "TypeScript".to_string(); repos[7].size_mb = 250; repos[7].last_commit_days = 2;
        repos[8].stars = 100_000; repos[8].forks = 25_000; repos[8].contributors = 1_600; repos[8].language = "JavaScript".to_string(); repos[8].size_mb = 400; repos[8].last_commit_days = 1;
        repos[9].stars = 65_000; repos[9].forks = 15_000; repos[9].contributors = 1_100; repos[9].language = "Go".to_string(); repos[9].size_mb = 200; repos[9].last_commit_days = 3;

        // Calculate complexity and classify
        for repo in &mut repos {
            repo.calculate_complexity();
            repo.classify_group_family();
        }

        // Create default view with all repositories
        let mut all_repos_view = RepositoryView::new("All Repositories".to_string(), "Complete mathematical analysis of all repositories".to_string());
        all_repos_view.repositories = repos;
        self.views.insert("all_repos".to_string(), all_repos_view);
    }

    fn create_view(&mut self, name: String, description: String) -> &mut RepositoryView {
        let view = RepositoryView::new(name.clone(), description);
        self.views.insert(view.name.clone(), view);
        self.views.get_mut(&name).unwrap()
    }

    fn share_view(&mut self, view_name: String, is_public: bool) -> Option<String> {
        if let Some(view) = self.views.get(&view_name) {
            let mut shared_view = view.clone();
            shared_view.shared_info = if is_public {
                format!("Public view shared via mathematical atlas")
            } else {
                format!("Private view - not shareable")
            };
            
            let paste_id = format!("paste_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            self.shared_views.insert(paste_id.clone(), shared_view);
            Some(paste_id)
        } else {
            None
        }
    }

    fn import_view(&mut self, paste_id: String) -> Option<&RepositoryView> {
        self.shared_views.get(&paste_id)
    }

    fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    fn list_shared_views(&self) -> Vec<String> {
        self.shared_views.keys().cloned().collect()
    }

    fn export_view(&self, view_name: String, output_path: &str) -> Result<(), String> {
        if let Some(view) = self.views.get(&view_name) {
            let content = view.generate_atlas_data();
            fs::write(output_path, content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            Ok(())
        } else {
            Err("View not found".to_string())
        }
    }

    fn generate_composition_json(&self, view_name: String) -> Result<String, String> {
        if let Some(view) = self.views.get(&view_name) {
            let mut composition = HashMap::new();
            
            composition.insert("view_name".to_string(), view.name.clone());
            composition.insert("view_description".to_string(), view.description.clone());
            
            // Family distribution
            let family_dist = view.get_family_distribution();
            composition.insert("family_distribution".to_string(), serde_json::to_string(&family_dist).unwrap());
            
            // Complexity statistics
            let (min_comp, max_comp, avg_comp) = view.get_complexity_stats();
            let mut complexity_stats = HashMap::new();
            complexity_stats.insert("minimum".to_string(), min_comp.to_string());
            complexity_stats.insert("maximum".to_string(), max_comp.to_string());
            complexity_stats.insert("average".to_string(), avg_comp.to_string());
            composition.insert("complexity_statistics".to_string(), serde_json::to_string(&complexity_stats).unwrap());
            
            // Repository details
            let mut repos_data = Vec::new();
            for repo in &view.repositories {
                let mut repo_data = HashMap::new();
                repo_data.insert("name".to_string(), repo.name.clone());
                repo_data.insert("path".to_string(), repo.path.clone());
                repo_data.insert("group_family".to_string(), repo.group_family.clone());
                repo_data.insert("complexity_score".to_string(), repo.complexity_score.to_string());
                repo_data.insert("stars".to_string(), repo.stars.to_string());
                repo_data.insert("forks".to_string(), repo.forks.to_string());
                repo_data.insert("contributors".to_string(), repo.contributors.to_string());
                repo_data.insert("language".to_string(), repo.language.clone());
                repo_data.insert("size_mb".to_string(), repo.size_mb.to_string());
                repo_data.insert("last_commit_days".to_string(), repo.last_commit_days.to_string());
                repo_data.insert("tile_size".to_string(), repo.get_tile_size().to_string());
                repo_data.insert("tile_color".to_string(), repo.get_tile_color());
                repos_data.push(repo_data);
            }
            composition.insert("repositories".to_string(), serde_json::to_string(&repos_data).unwrap());
            
            Ok(serde_json::to_string_pretty(&composition).unwrap())
        } else {
            Err("View not found".to_string())
        }
    }
}

fn main() {
    println!("🎯 Repository Mathematical Atlas - Analyzing Git Repositories Through Group Theory\n");
    
    let mut composer = RepositoryAtlasComposer::new();
    
    // Create sample repositories
    println!("📊 Creating sample repositories with mathematical classification...");
    composer.create_sample_repositories();
    
    // Create specialized views
    println!("🔍 Creating specialized mathematical views...");
    
    // Cyclic repositories view
    let mut cyclic_view = composer.create_view(
        "Cyclic Repositories".to_string(),
        "Repositories classified as Cyclic group family".to_string()
    );
    cyclic_view.filter_by_family("Cyclic".to_string());
    
    // High complexity view
    let mut high_complexity_view = composer.create_view(
        "High Complexity Repositories".to_string(),
        "Repositories with complexity score > 2.5".to_string()
    );
    high_complexity_view.filter_by_complexity(2.5, 5.0);
    
    // JavaScript repositories view
    let mut js_view = composer.create_view(
        "JavaScript Repositories".to_string(),
        "JavaScript language repositories".to_string()
    );
    js_view.filter_by_language("JavaScript".to_string());
    
    // Star repositories view
    let mut star_view = composer.create_view(
        "Star Repositories".to_string(),
        "Repositories with > 100,000 stars".to_string()
    );
    star_view.filter_by_stars(Some(100_000), None);
    
    // List all views
    println!("\n📋 Available Views:");
    for view_name in composer.list_views() {
        if let Some(view) = composer.views.get(&view_name) {
            println!("  - {}: {} ({} repositories)", view.name, view.description, view.repositories.len());
        }
    }
    
    // Generate and export compositions
    println!("\n📤 Generating mathematical compositions...");
    
    let views_to_export = vec!["all_repos", "Cyclic Repositories", "High Complexity Repositories", "JavaScript Repositories", "Star Repositories"];
    
    for view_name in views_to_export {
        match composer.generate_composition_json(view_name.to_string()) {
            Ok(json_content) => {
                let output_path = format!("./repository_atlas_output/repository_composition_{}.json", view_name.replace(" ", "_").to_lowercase());
                fs::write(&output_path, json_content).unwrap();
                println!("✅ Generated: {}", output_path);
            }
            Err(e) => println!("❌ Failed to generate {}: {}", view_name, e),
        }
    }
    
    // Share views
    println!("\n🌐 Sharing views via mathematical atlas...");
    
    if let Some(paste_id) = composer.share_view("Cyclic Repositories".to_string(), true) {
        println!("📤 Shared 'Cyclic Repositories' as paste ID: {}", paste_id);
    }
    
    if let Some(paste_id) = composer.share_view("High Complexity Repositories".to_string(), true) {
        println!("📤 Shared 'High Complexity Repositories' as paste ID: {}", paste_id);
    }
    
    // List shared views
    println!("\n📋 Shared Views:");
    for shared_id in composer.list_shared_views() {
        println!("  - {}", shared_id);
    }
    
    // Import and display shared view
    println!("\n📥 Importing shared views...");
    if let Some(shared_view) = composer.import_view("paste_".to_string() + &SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()) {
        println!("✅ Successfully imported shared view with {} repositories", shared_view.repositories.len());
    }
    
    // Export atlas data
    println!("\n📚 Exporting complete atlas data...");
    if let Some(all_repos) = composer.views.get("all_repos") {
        let atlas_content = all_repos.generate_atlas_data();
        fs::write("./repository_atlas_output/complete_atlas.md", atlas_content).unwrap();
        println!("✅ Complete atlas exported to: ./repository_atlas_output/complete_atlas.md");
    }
    
    println!("\n🎉 Repository Mathematical Atlas Analysis Complete!");
    println!("📁 Check the ./repository_atlas_output/ directory for generated files.");
}