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

    fn generate_view_data(&self) -> String {
        let mut data = String::new();
        
        data.push_str(&format!("# Repository Analysis: {}\\n\\n", self.name));
        data.push_str(&format!("**Description**: {}\\n\\n", self.description));
        
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
        repos[1].stars = 150_000; repos[1].forks = 50_000; repos[1].contributors = 2_500; repos[1].language = "C".to_string(); repos[1].size_mb = 1000; repos[1].last_commit_days = 0;
        repos[2].stars = 150_000; repos[2].forks = 25_000; repos[2].contributors = 1_800; repos[2].language = "TypeScript".to_string(); repos[2].size_mb = 300; repos[2].last_commit_days = 1;
        repos[3].stars = 200_000; repos[3].forks = 40_000; repos[3].contributors = 2_200; repos[3].language = "JavaScript".to_string(); repos[3].size_mb = 200; repos[3].last_commit_days = 0;
        repos[4].stars = 170_000; repos[4].forks = 70_000; repos[4].contributors = 5_000; repos[4].language = "Python".to_string(); repos[4].size_mb = 800; repos[4].last_commit_days = 2;

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
    println!("ℹ️  Note: This tool analyzes repository metrics (stars, forks, language, etc.)");
    println!("   Mathematical work on finite simple groups is handled in associated Lean 4 formalization.");
}