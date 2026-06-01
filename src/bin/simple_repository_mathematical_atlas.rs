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
    // Actual code metrics that can be measured
    lines_of_code: Option<usize>,
    commit_frequency: Option<f64>, // commits per week
    issue_close_rate: Option<f64>, // issues closed per issue opened
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
            lines_of_code: None,
            commit_frequency: None,
            issue_close_rate: None,
        }
    }

    // These would be implemented with actual data collection in a real tool
    fn set_lines_of_code(&mut self, loc: usize) {
        self.lines_of_code = Some(loc);
    }

    fn set_commit_frequency(&mutely, freq: f64) {
        self.commit_frequency = Some(freq);
    }

    fn set_issue_close_rate(&mut self, rate: f64) {
        self.issue_close_rate = Some(rate);
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

    fn get_language_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for repo in &self.repositories {
            *distribution.entry(repo.language.clone()).or_insert(0) += 1;
        }
        distribution
    }

    fn get_activity_stats(&self) -> (f64, f64, f64) {
        let mut frequencies = Vec::new();
        for repo in &self.repositories {
            if let Some(freq) = repo.commit_frequency {
                frequencies.push(freq);
            }
        }
        
        if frequencies.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let min = *frequencies.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = *frequencies.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg = frequencies.iter().sum::<f64>() / frequencies.len() as f64;
        
        (min, max, avg)
    }

    fn generate_view_data(&self) -> String {
        let mut data = String::new();
        
        data.push_str(&format!("# Repository Analysis: {}\\n\\n", self.name));
        data.push_str(&format!("**Description**: {}\\n\\n", self.description));
        
        // Language distribution
        let lang_dist = self.get_language_distribution();
        if !lang_dist.is_empty() {
            data.push_str("## Language Distribution\\n\\n");
            let mut sorted: Vec<(&String, &usize)> = lang_dist.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (lang, count) in sorted {
                data.push_str(&format!("- **{}**: {} repositories\\n", lang, count));
            }
            data.push_str("\\n");
        }
        
        // Activity statistics (if available)
        let (min_freq, max_freq, avg_freq) = self.get_activity_stats();
        if avg_freq > 0.0 {
            data.push_str("## Activity Statistics (commits/week)\\n\\n");
            data.push_str(&format!("- **Minimum**: {:.2}\\n", min_freq));
            data.push_str(&format!("- **Maximum**: {:.2}\\n", max_freq));
            data.push_str(&format!("- **Average**: {:.2}\\n", avg_freq));
            data.push_str("\\n");
        }
        
        data.push_str("## Repository Details\\n\\n");
        for (i, repo) in self.repositories.iter().enumerate() {
            data.push_str(&format!("### {}. {}\\n", i + 1, repo.name));
            data.push_str(&format!("- **Path**: {}\\n", repo.path));
            data.push_str(&format!("- **Language**: {}\\n", repo.language));
            data.push_str(&format!("- **Stars**: {}\\n", repo.stars));
            data.push_str(&format!("- **Forks**: {}\\n", repo.forks));
            data.push_str(&format!("- **Contributors**: {}\\n", repo.contributors));
            data.push_str(&format!("- **Is Fork**: {}\\n", if repo.is_fork { "Yes" } else { "No" }));
            data.push_str(&format!("- **Size**: {} MB\\n", repo.size_mb));
            data.push_str(&format!("- **Last Commit**: {} days ago\\n", repo.last_commit_days));
            
            if let Some(loc) = repo.lines_of_code {
                data.push_str(&format!("- **Lines of Code**: {}\\n", loc));
            }
            
            if let Some(freq) = repo.commit_frequency {
                data.push_str(&format!("- **Commit Frequency**: {:.2} commits/week\\n", freq));
            }
            
            if let Some(rate) = repo.issue_close_rate {
                data.push_str(&format!("- **Issue Close Rate**: {:.2}\\n", rate));
            }
            
            data.push_str("\\n");
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

        // Assign some realistic values for demonstration
        repos[0].stars = 75_000; repos[0].forks = 12_000; repos[0].contributors = 1_200; repos[0].language = "Rust".to_string(); repos[0].size_mb = 500; repos[0].last_commit_days = 1;
        repos[0].set_lines_of_code(6_000_000); repos[0].set_commit_frequency(150.0); repos[0].set_issue_close_rate(0.8);
        
        repos[1].stars = 150_000; repos[1].forks = 50_000; repos[1].contributors = 2_500; repos[1].language = "C".to_string(); repos[1].size_mb = 1000; repos[1].last_commit_days = 0;
        repos[1].set_lines_of_code(30_000_000); repos[1].set_commit_frequency(200.0); repos[1].set_issue_close_rate(0.7);
        
        repos[2].stars = 150_000; repos[2].forks = 25_000; repos[2].contributors = 1_800; repos[2].language = "TypeScript".to_string(); repos[2].size_mb = 300; repos[2].last_commit_days = 1;
        repos[2].set_lines_of_code(800_000); repos[2].set_commit_frequency(75.0); repos[2].set_issue_close_rate(0.85);
        
        repos[3].stars = 200_000; repos[3].forks = 40_000; repos[3].contributors = 2_200; repos[3].language = "JavaScript".to_string(); repos[3].size_mb = 200; repos[3].last_commit_days = 0;
        repos[3].set_lines_of_code(500_000); repos[3].set_commit_frequency(100.0); repos[3].set_issue_close_rate(0.9);
        
        repos[4].stars = 170_000; repos[4].forks = 70_000; repos[4].contributors = 5_000; repos[4].language = "Python".to_string(); repos[4].size_mb = 800; repos[4].last_commit_days = 2;
        repos[4].set_lines_of_code(400_000); repos[4].set_commit_frequency(50.0); repos[4].set_issue_close_rate(0.75);

        let mut all_repos_view = RepositoryView::new("All Repositories".to_string(), "Analysis of popular open-source repositories".to_string());
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
            
            // Language distribution
            let lang_dist = view.get_language_distribution();
            composition.insert("language_distribution".to_string(), format!("{:?}", lang_dist));
            
            // Activity stats
            let (min_freq, max_freq, avg_freq) = view.get_activity_stats();
            let mut activity_stats = HashMap::new();
            activity_stats.insert("min_commits_per_week".to_string(), min_freq.to_string());
            activity_stats.insert("max_commits_per_week".to_string(), max_freq.to_string());
            activity_stats.insert("avg_commits_per_week".to_string(), avg_freq.to_string());
            composition.insert("activity_statistics".to_string(), format!("{:?}", activity_stats));
            
            let mut repos_data = Vec::new();
            for repo in &view.repositories {
                let mut repo_data = HashMap::new();
                repo_data.insert("name".to_string(), repo.name.clone());
                repo_data.insert("path".to_string(), repo.path.clone());
                repo_data.insert("language".to_string(), repo.language.clone());
                repo_data.insert("stars".to_string(), repo.stars.to_string());
                repo_data.insert("forks".to_string(), repo.forks.to_string());
                repo_data.insert("contributors".to_string(), repo.contributors.to_string());
                repo_data.insert("is_fork".to_string(), repo.is_fork.to_string());
                repo_data.insert("size_mb".to_string(), repo.size_mb.to_string());
                repo_data.insert("last_commit_days".to_string(), repo.last_commit_days.to_string());
                
                if let Some(loc) = repo.lines_of_code {
                    repo_data.insert("lines_of_code".to_string(), loc.to_string());
                }
                
                if let Some(freq) = repo.commit_frequency {
                    repo_data.insert("commit_frequency".to_string(), freq.to_string());
                }
                
                if let Some(rate) = repo.issue_close_rate {
                    repo_data.insert("issue_close_rate".to_string(), rate.to_string());
                }
                
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
    println!("📊 Repository Analysis Tool - Analyzing Popular Open-Source Projects\\n");
    
    let mut composer = RepositoryAtlasComposer::new();
    
    println!("📋 Creating sample repository data for analysis...");
    composer.create_sample_repositories();
    
    println!("\\n📋 Available Analysis Views:");
    for view_name in composer.views.keys() {
        if let Some(view) = composer.views.get(view_name) {
            println!("  - {}: {} ({} repositories)", view.name, view.description, view.repositories.len());
        }
    }
    
    println!("\\n📤 Generating repository analyses...");
    
    let views_to_export = vec!["all_repos"];
    
    for view_name in views_to_export {
        match composer.generate_composition_json(view_name.to_string()) {
            Ok(json_content) => {
                let output_path = format!("./repository_atall_output/repository_analysis_{}.json", view_name.replace(" ", "_").to_lowercase());
                // Fix directory name
                let _ = fs::create_dir_all("./repository_atlas_output");
                let output_path = format!("./repository_atlas_output/repository_analysis_{}.json", view_name.replace(" ", "_").to_lowercase());
                fs::write(&output_path, json_content).unwrap();
                println!("✅ Generated: {}", output_path);
            }
            Err(e) => println!("❌ Failed to generate {}: {}", view_name, e),
        }
    }
    
    println!("\\n📚 Exporting complete analysis data...");
    if let Some(all_repos) = composer.views.get("all_repos") {
        let analysis_content = all_repos.generate_view_data();
        let _ = fs::create_dir_all("./repository_atlas_output");
        fs::write("./repository_atlas_output/complete_repository_analysis.md", analysis_content).unwrap();
        println!("✅ Complete analysis exported to: ./repository_atlas_output/complete_repository_analysis.md");
    }
    
    println!("\\n🎉 Repository Analysis Complete!");
    println!("📁 Check the ./repository_atlas_output/ directory for generated files.");
    println!("ℹ️  Note: This tool analyzes actual repository metrics (stars, forks, language, etc.)");
    println!("   It does not make any claims about mathematical group theory classification.");
}