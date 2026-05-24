use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// Enhanced Repository structure with mathematical group theory concepts
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
    group_order: String,
    group_rank: usize,
    automorphism_group_size: usize,
    simple_subgroups: Vec<String>,
    composition_factors: Vec<String>,
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
            group_order: "1".to_string(),
            group_rank: 0,
            automorphism_group_size: 1,
            simple_subgroups: Vec::new(),
            composition_factors: Vec::new(),
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
        
        if self.has_issues { complexity += 0.05; }
        if self.has_wiki { complexity += 0.05; }
        
        self.complexity_score = complexity;
    }

    fn classify_group_family(&mut self) {
        if self.is_fork {
            self.group_family = "Cyclic".to_string();
            self.group_order = "2".to_string();
            self.group_rank = 1;
            self.mathematical_properties.insert("order".to_string(), "2".to_string());
            self.mathematical_properties.insert("type".to_string(), "fork".to_string());
            self.composition_factors = vec!["C2".to_string()];
        } else if self.complexity_score < 1.0 {
            let prime_order = self.get_prime_order();
            self.group_family = "Cyclic".to_string();
            self.group_order = prime_order.to_string();
            self.group_rank = 1;
            self.mathematical_properties.insert("order".to_string(), prime_order.to_string());
            self.mathematical_properties.insert("type".to_string(), "simple".to_string());
            self.composition_factors = vec![format!("C{}", prime_order)];
        } else if self.complexity_score < 2.0 {
            self.group_family = "Alternating".to_string();
            let n = (self.complexity_score * 5.0).ceil() as usize;
            self.group_order = self.alternating_order(n).to_string();
            self.group_rank = n - 2;
            self.mathematical_properties.insert("order".to_string(), self.group_order.to_string());
            self.mathematical_properties.insert("type".to_string(), "alternating".to_string());
            self.composition_factors = vec![format!("A{}", n)];
        } else if self.complexity_score < 3.0 {
            self.group_family = "Lie Type".to_string();
            let rank = ((self.complexity_score - 2.0) * 3.0).ceil() as usize;
            let field_size = ((self.complexity_score - 2.0) * 2.0 + 2.0).ceil() as usize;
            self.group_order = self.lie_type_order(rank, field_size).to_string();
            self.group_rank = rank;
            self.mathematical_properties.insert("order".to_string(), self.group_order.to_string());
            self.mathematical_properties.insert("type".to_string(), "lie".to_string());
            self.mathematical_properties.insert("rank".to_string(), rank.to_string());
            self.mathematical_properties.insert("field".to_string(), field_size.to_string());
            self.composition_factors = vec![format!("PSL({}, {})", rank, field_size)];
        } else if self.complexity_score < 4.0 {
            self.group_family = "Sporadic".to_string();
            let sporadic_type = self.get_sporadic_type();
            self.group_order = self.sporadic_order(&sporadic_type).to_string();
            self.group_rank = 0;
            self.mathematical_properties.insert("order".to_string(), self.group_order.to_string());
            self.mathematical_properties.insert("type".to_string(), "sporadic".to_string());
            self.mathematical_properties.insert("name".to_string(), sporadic_type.clone());
            self.composition_factors = vec![sporadic_type];
        } else {
            self.group_family = "Twisted Lie".to_string();
            self.group_order = "808017424794512875886459904961710757005754368000000000".to_string(); // Monster group order
            self.group_rank = 2;
            self.mathematical_properties.insert("order".to_string(), "8.08×10⁵³".to_string());
            self.mathematical_properties.insert("type".to_string(), "twisted".to_string());
            self.mathematical_properties.insert("name".to_string(), "Monster".to_string());
            self.composition_factors = vec!["Monster".to_string()];
        }
        
        self.automorphism_group_size = self.group_order.parse::<usize>().unwrap_or(1) / 2;
        self.find_simple_subgroups();
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

    fn lie_type_order(&self, rank: usize, field_size: usize) -> usize {
        let mut order = 1;
        for _i in 1..=rank {
            order *= field_size.pow((rank * (rank - 1) / 2) as u32);
        }
        order * 1000
    }

    fn get_sporadic_type(&self) -> String {
        let sporadic_types = vec![
            "Mathieu M11", "Mathieu M12", "Mathieu M22", "Mathieu M23", "Mathieu M24",
            "Janko J1", "Janko J2", "Janko J3", "Janko J4",
            "Conway Co1", "Conway Co2", "Conway Co3",
            "Fischer Fi22", "Fischer Fi23", "Fischer Fi24'",
            "Held He", "McL", "Suz", "Ru", "O'Nan", "HS", "Ly", "Th", "HN",
        ];
        let index = ((self.complexity_score - 3.0) * sporadic_types.len() as f64).min(sporadic_types.len() as f64 - 1.0) as usize;
        sporadic_types[index].to_string()
    }

    fn sporadic_order(&self, sporadic_type: &str) -> String {
        match sporadic_type {
            "Mathieu M11" => "7920".to_string(),
            "Mathieu M12" => "95040".to_string(),
            "Mathieu M22" => "443520".to_string(),
            "Mathieu M23" => "10200960".to_string(),
            "Mathieu M24" => "244823040".to_string(),
            "Janko J1" => "175560".to_string(),
            "Janko J2" => "604800".to_string(),
            "Janko J3" => "50232960".to_string(),
            "Janko J4" => "867725561600".to_string(),
            "Conway Co1" => "4157776806543360000".to_string(),
            "Conway Co2" => "42305421312000".to_string(),
            "Conway Co3" => "495766656000".to_string(),
            "Fischer Fi22" => "64561751654400".to_string(),
            "Fischer Fi23" => "4089470473293004800".to_string(),
            "Fischer Fi24'" => "1255205709190661721292800".to_string(),
            "Held He" => "4030387200".to_string(),
            "McL" => "898128000".to_string(),
            "Suz" => "448345497600".to_string(),
            "Ru" => "145926144000".to_string(),
            "O'Nan" => "460815505920".to_string(),
            "HS" => "44352000".to_string(),
            "Ly" => "51765179004000000".to_string(),
            "Th" => "90745943887872000".to_string(),
            "HN" => "273030912000000".to_string(),
            _ => "1000000".to_string(),
        }
    }

    fn find_simple_subgroups(&mut self) {
        match self.group_family.as_str() {
            "Cyclic" => {
                if self.group_order.parse::<usize>().unwrap_or(1) > 1 {
                    self.simple_subgroups = vec![format!("C{}", self.group_order)];
                }
            },
            "Alternating" => {
                let n = ((self.complexity_score * 5.0).ceil() as usize).max(5);
                for i in (5..=n).step_by(2) {
                    self.simple_subgroups.push(format!("A{}", i));
                }
            },
            "Lie Type" => {
                self.simple_subgroups = vec![format!("PSL({}, {})", self.group_rank, 2)];
            },
            "Sporadic" => {
                self.simple_subgroups = vec![self.mathematical_properties.get("name").unwrap_or(&"Unknown".to_string()).clone()];
            },
            "Twisted Lie" => {
                self.simple_subgroups = vec!["Monster".to_string(), "Baby Monster".to_string()];
            },
            _ => {}
        }
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
            "Twisted Lie" => "#FFEAA7".to_string(),
            _ => "#DDA0DD".to_string(),
        }
    }

    fn get_group_theory_description(&self) -> String {
        match self.group_family.as_str() {
            "Cyclic" => format!("Cyclic group of order {} - Prime order simple group", self.group_order),
            "Alternating" => format!("Alternating group A_{} - Non-abelian simple group", self.group_rank + 2),
            "Lie Type" => format!("Lie type group PSL({}, {}) - Classical simple group", self.group_rank, self.mathematical_properties.get("field").unwrap_or(&"2".to_string())),
            "Sporadic" => format!("Sporadic group {} - Exceptional simple group", self.mathematical_properties.get("name").unwrap_or(&"Unknown".to_string())),
            "Twisted Lie" => format!("Twisted Lie type group {} - Exceptional simple group", self.mathematical_properties.get("name").unwrap_or(&"Monster".to_string())),
            _ => "Unclassified group".to_string(),
        }
    }
}

// Enhanced Repository View with advanced mathematical analysis
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
    mathematical_analysis: HashMap<String, String>,
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
            mathematical_analysis: HashMap::new(),
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
        let mut filtered_repos = Vec::new();
        for repo in &self.repositories {
            let mut include = true;
            
            if let Some(ref family) = self.family_filter {
                if repo.group_family != *family {
                    include = false;
                }
            }

            if let Some((min, max)) = self.complexity_range {
                if repo.complexity_score < min || repo.complexity_score > max {
                    include = false;
                }
            }

            if let Some(ref language) = self.language_filter {
                if repo.language != *language {
                    include = false;
                }
            }

            if let Some(min) = self.min_stars {
                if repo.stars < min {
                    include = false;
                }
            }

            if let Some(max) = self.max_stars {
                if repo.stars > max {
                    include = false;
                }
            }

            if include {
                filtered_repos.push(repo.clone());
            }
        }
        self.repositories = filtered_repos;
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

    fn get_order_statistics(&self) -> (usize, usize, f64) {
        if self.repositories.is_empty() {
            return (0, 0, 0.0);
        }
        
        let orders: Vec<usize> = self.repositories.iter().map(|r| r.group_order.parse::<usize>().unwrap_or(1)).collect();
        let min = orders.iter().fold(usize::MAX, |a, &b| a.min(b));
        let max = orders.iter().fold(0, |a, &b| a.max(b));
        let avg = orders.iter().sum::<usize>() as f64 / orders.len() as f64;
        
        (min, max, avg)
    }

    fn analyze_group_theory_properties(&mut self) {
        let mut total_order = 0;
        let mut total_rank = 0;
        let mut total_automorphisms = 0;
        let mut all_simple_subgroups = Vec::new();
        
        for repo in &self.repositories {
            total_order += repo.group_order.parse::<usize>().unwrap_or(1);
            total_rank += repo.group_rank;
            total_automorphisms += repo.automorphism_group_size;
            all_simple_subgroups.extend(&repo.simple_subgroups);
        }
        
        all_simple_subgroups.sort();
        all_simple_subgroups.dedup();
        
        self.mathematical_analysis.insert("total_group_order".to_string(), total_order.to_string());
        self.mathematical_analysis.insert("average_group_order".to_string(), (total_order as f64 / self.repositories.len() as f64).to_string());
        self.mathematical_analysis.insert("total_rank".to_string(), total_rank.to_string());
        self.mathematical_analysis.insert("average_rank".to_string(), (total_rank as f64 / self.repositories.len() as f64).to_string());
        self.mathematical_analysis.insert("total_automorphisms".to_string(), total_automorphisms.to_string());
        self.mathematical_analysis.insert("unique_simple_subgroups".to_string(), all_simple_subgroups.len().to_string());
        self.mathematical_analysis.insert("simple_subgroups".to_string(), format!("{:?}", all_simple_subgroups));
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

        let (min_order, max_order, avg_order) = self.get_order_statistics();
        data.push_str("## Group Order Statistics\n\n");
        data.push_str(&format!("- **Minimum Group Order**: {}\n", min_order));
        data.push_str(&format!("- **Maximum Group Order**: {}\n", max_order));
        data.push_str(&format!("- **Average Group Order**: {:.0}\n", avg_order));
        data.push_str("\n");

        data.push_str("## Mathematical Analysis\n\n");
        for (key, value) in &self.mathematical_analysis {
            data.push_str(&format!("- **{}**: {}\n", key, value));
        }
        data.push_str("\n");

        data.push_str("## Repository Details\n\n");
        for (i, repo) in self.repositories.iter().enumerate() {
            data.push_str(&format!("### {}. {}\n", i + 1, repo.name));
            data.push_str(&format!("- **Path**: {}\n", repo.path));
            data.push_str(&format!("- **Group Family**: {}\n", repo.group_family));
            data.push_str(&format!("- **Group Order**: {}\n", repo.group_order));
            data.push_str(&format!("- **Group Rank**: {}\n", repo.group_rank));
            data.push_str(&format!("- **Complexity Score**: {:.2}\n", repo.complexity_score));
            data.push_str(&format!("- **Automorphism Group Size**: {}\n", repo.automorphism_group_size));
            data.push_str(&format!("- **Simple Subgroups**: {}\n", repo.simple_subgroups.join(", ")));
            data.push_str(&format!("- **Composition Factors**: {}\n", repo.composition_factors.join(", ")));
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

// Enhanced Repository Atlas Composer
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
            Repository::new("golang/go".to_string(), "/go/go".to_string()),
            Repository::new("apple/swift".to_string(), "/swift/swift".to_string()),
            Repository::new("apache/kafka".to_string(), "/kafka/kafka".to_string()),
            Repository::new("elastic/elasticsearch".to_string(), "/elasticsearch/elasticsearch".to_string()),
            Repository::new("mongodb/mongo".to_string(), "/mongo/mongo".to_string()),
        ];

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
        repos[10].stars = 120_000; repos[10].forks = 20_000; repos[10].contributors = 1_700; repos[10].language = "Go".to_string(); repos[10].size_mb = 300; repos[10].last_commit_days = 1;
        repos[11].stars = 70_000; repos[11].forks = 12_000; repos[11].contributors = 1_400; repos[11].language = "Swift".to_string(); repos[11].size_mb = 250; repos[11].last_commit_days = 2;
        repos[12].stars = 25_000; repos[12].forks = 10_000; repos[12].contributors = 800; repos[12].language = "Java".to_string(); repos[12].size_mb = 150; repos[12].last_commit_days = 5;
        repos[13].stars = 65_000; repos[13].forks = 25_000; repos[13].contributors = 1_200; repos[13].language = "Java".to_string(); repos[13].size_mb = 400; repos[13].last_commit_days = 3;
        repos[14].stars = 20_000; repos[14].forks = 5_000; repos[14].contributors = 600; repos[14].language = "C++".to_string(); repos[14].size_mb = 100; repos[14].last_commit_days = 7;

        for repo in &mut repos {
            repo.calculate_complexity();
            repo.classify_group_family();
        }

        let mut all_repos_view = RepositoryView::new("All Repositories".to_string(), "Complete mathematical analysis of all repositories".to_string());
        all_repos_view.repositories = repos;
        all_repos_view.analyze_group_theory_properties();
        self.views.insert("all_repos".to_string(), all_repos_view);
    }

    fn create_view(&mut self, name: String, description: String) -> &mut RepositoryView {
        let view = RepositoryView::new(name.clone(), description);
        self.views.insert(name.clone(), view);
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
            
            let family_dist = self.get_family_distribution_for_view(view);
            composition.insert("family_distribution".to_string(), format!("{:?}", family_dist));
            
            let (min_comp, max_comp, avg_comp) = view.get_complexity_stats();
            let mut complexity_stats = HashMap::new();
            complexity_stats.insert("minimum".to_string(), min_comp.to_string());
            complexity_stats.insert("maximum".to_string(), max_comp.to_string());
            complexity_stats.insert("average".to_string(), avg_comp.to_string());
            composition.insert("complexity_statistics".to_string(), format!("{:?}", complexity_stats));
            
            let (min_order, max_order, avg_order) = view.get_order_statistics();
            let mut order_stats = HashMap::new();
            order_stats.insert("minimum".to_string(), min_order.to_string());
            order_stats.insert("maximum".to_string(), max_order.to_string());
            order_stats.insert("average".to_string(), avg_order.to_string());
            composition.insert("order_statistics".to_string(), format!("{:?}", order_stats));
            
            composition.insert("mathematical_analysis".to_string(), format!("{:?}", view.mathematical_analysis));
            
            let mut repos_data = Vec::new();
            for repo in &view.repositories {
                let mut repo_data = HashMap::new();
                repo_data.insert("name".to_string(), repo.name.clone());
                repo_data.insert("path".to_string(), repo.path.clone());
                repo_data.insert("group_family".to_string(), repo.group_family.clone());
                repo_data.insert("group_order".to_string(), repo.group_order.clone());
                repo_data.insert("group_rank".to_string(), repo.group_rank.to_string());
                repo_data.insert("complexity_score".to_string(), repo.complexity_score.to_string());
                repo_data.insert("automorphism_group_size".to_string(), repo.automorphism_group_size.to_string());
                repo_data.insert("simple_subgroups".to_string(), format!("{:?}", repo.simple_subgroups));
                repo_data.insert("composition_factors".to_string(), format!("{:?}", repo.composition_factors));
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

    fn get_family_distribution_for_view(&self, view: &RepositoryView) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for repo in &view.repositories {
            *distribution.entry(repo.group_family.clone()).or_insert(0) += 1;
        }
        distribution
    }
}

fn main() {
    println!("🎯 Enhanced Repository Mathematical Atlas - Advanced Group Theory Analysis\n");
    
    let mut composer = RepositoryAtlasComposer::new();
    
    println!("📊 Creating enhanced repositories with detailed mathematical classification...");
    composer.create_sample_repositories();
    
    println!("🔍 Creating specialized mathematical views...");
    
    let cyclic_view = composer.create_view(
        "Cyclic Repositories".to_string(),
        "Repositories classified as Cyclic group family".to_string()
    );
    cyclic_view.filter_by_family("Cyclic".to_string());
    cyclic_view.analyze_group_theory_properties();
    
    let high_complexity_view = composer.create_view(
        "High Complexity Repositories".to_string(),
        "Repositories with complexity score > 2.5".to_string()
    );
    high_complexity_view.filter_by_complexity(2.5, 5.0);
    high_complexity_view.analyze_group_theory_properties();
    
    let js_view = composer.create_view(
        "JavaScript Repositories".to_string(),
        "JavaScript language repositories".to_string()
    );
    js_view.filter_by_language("JavaScript".to_string());
    js_view.analyze_group_theory_properties();
    
    let star_view = composer.create_view(
        "Star Repositories".to_string(),
        "Repositories with > 100,000 stars".to_string()
    );
    star_view.filter_by_stars(Some(100_000), None);
    star_view.analyze_group_theory_properties();
    
    let lie_view = composer.create_view(
        "Lie Type Repositories".to_string(),
        "Repositories classified as Lie Type group family".to_string()
    );
    lie_view.filter_by_family("Lie Type".to_string());
    lie_view.analyze_group_theory_properties();
    
    println!("\n📋 Available Views:");
    for view_name in composer.list_views() {
        if let Some(view) = composer.views.get(&view_name) {
            println!("  - {}: {} ({} repositories)", view.name, view.description, view.repositories.len());
        }
    }
    
    println!("\n📤 Generating enhanced mathematical compositions...");
    
    let views_to_export = vec![
        "all_repos", 
        "Cyclic Repositories", 
        "High Complexity Repositories", 
        "JavaScript Repositories", 
        "Star Repositories",
        "Lie Type Repositories"
    ];
    
    for view_name in views_to_export {
        match composer.generate_composition_json(view_name.to_string()) {
            Ok(json_content) => {
                let output_path = format!("./repository_atlas_output/enhanced_composition_{}.json", view_name.replace(" ", "_").to_lowercase());
                fs::write(&output_path, json_content).unwrap();
                println!("✅ Generated: {}", output_path);
            }
            Err(e) => println!("❌ Failed to generate {}: {}", view_name, e),
        }
    }
    
    println!("\n🌐 Sharing views via enhanced mathematical atlas...");
    
    if let Some(paste_id) = composer.share_view("Cyclic Repositories".to_string(), true) {
        println!("📤 Shared 'Cyclic Repositories' as paste ID: {}", paste_id);
    }
    
    if let Some(paste_id) = composer.share_view("High Complexity Repositories".to_string(), true) {
        println!("📤 Shared 'High Complexity Repositories' as paste ID: {}", paste_id);
    }
    
    if let Some(paste_id) = composer.share_view("Lie Type Repositories".to_string(), true) {
        println!("📤 Shared 'Lie Type Repositories' as paste ID: {}", paste_id);
    }
    
    println!("\n📋 Shared Views:");
    for shared_id in composer.list_shared_views() {
        println!("  - {}", shared_id);
    }
    
    println!("\n📥 Importing shared views...");
    if let Some(shared_view) = composer.import_view("paste_".to_string() + &SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()) {
        println!("✅ Successfully imported shared view with {} repositories", shared_view.repositories.len());
    }
    
    println!("\n📚 Exporting complete enhanced atlas data...");
    if let Some(all_repos) = composer.views.get("all_repos") {
        let atlas_content = all_repos.generate_atlas_data();
        fs::write("./repository_atlas_output/enhanced_complete_atlas.md", atlas_content).unwrap();
        println!("✅ Complete enhanced atlas exported to: ./repository_atlas_output/enhanced_complete_atlas.md");
    }
    
    println!("\n🎉 Enhanced Repository Mathematical Atlas Analysis Complete!");
    println!("📁 Check the ./repository_atlas_output/ directory for generated files.");
}