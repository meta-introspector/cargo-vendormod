#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test helper function to create a test repository
    fn create_test_repo(name: &str, stars: usize, forks: usize, contributors: usize, language: &str, size_mb: usize, last_commit_days: usize) -> Repository {
        let mut repo = Repository::new(name.to_string(), format!("/test/{}", name));
        repo.stars = stars;
        repo.forks = forks;
        repo.contributors = contributors;
        repo.language = language.to_string();
        repo.size_mb = size_mb;
        repo.last_commit_days = last_commit_days;
        repo
    }

    #[test]
    fn test_repository_creation() {
        let repo = Repository::new("test/repo".to_string(), "/test/repo".to_string());
        
        assert_eq!(repo.name, "test/repo");
        assert_eq!(repo.path, "/test/repo");
        assert_eq!(repo.stars, 0);
        assert_eq!(repo.forks, 0);
        assert_eq!(repo.contributors, 0);
        assert_eq!(repo.language, "Unknown");
        assert_eq!(repo.size_mb, 0);
        assert_eq!(repo.last_commit_days, 0);
        assert_eq!(repo.complexity_score, 0.0);
        assert_eq!(repo.group_family, "Unclassified");
        assert_eq!(repo.group_order, "1");
        assert_eq!(repo.group_rank, 0);
        assert!(repo.simple_subgroups.is_empty());
    }

    #[test]
    fn test_complexity_calculation_low() {
        let mut repo = create_test_repo("low_complexity", 10, 5, 3, "Rust", 10, 30);
        repo.calculate_complexity();
        
        // Low values should result in low complexity
        assert!(repo.complexity_score > 0.0);
        assert!(repo.complexity_score < 1.0);
    }

    #[test]
    fn test_complexity_calculation_high() {
        let mut repo = create_test_repo("high_complexity", 100000, 50000, 1000, "Python", 1000, 1);
        repo.calculate_complexity();
        
        // High values should result in high complexity
        assert!(repo.complexity_score > 2.0);
    }

    #[test]
    fn test_complexity_calculation_zero_activity() {
        let mut repo = create_test_repo("no_activity", 1000, 500, 100, "JavaScript", 100, 0);
        repo.calculate_complexity();
        
        // Zero activity days should not cause division by zero
        assert!(repo.complexity_score > 0.0);
    }

    #[test]
    fn test_group_classification_cyclic() {
        let mut repo = create_test_repo("cyclic_repo", 5, 2, 1, "C", 10, 100);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Low complexity should result in Cyclic classification
        assert_eq!(repo.group_family, "Cyclic");
        assert_eq!(repo.group_order, "2");
        assert_eq!(repo.group_rank, 1);
        assert_eq!(repo.simple_subgroups, vec!["C2"]);
    }

    #[test]
    fn test_group_classification_alternating() {
        let mut repo = create_test_repo("alternating_repo", 1000, 500, 100, "C++", 100, 50);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Medium complexity should result in Alternating classification
        assert_eq!(repo.group_family, "Alternating");
        assert_eq!(repo.group_rank, 3); // A5
    }

    #[test]
    fn test_group_classification_lie_type() {
        let mut repo = create_test_repo("lie_type_repo", 100000, 50000, 1000, "Python", 1000, 10);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // High complexity should result in Lie Type classification
        assert_eq!(repo.group_family, "Lie Type");
        assert_eq!(repo.group_rank, 3);
    }

    #[test]
    fn test_group_classification_sporadic() {
        let mut repo = create_test_repo("sporadic_repo", 200000, 100000, 2000, "Java", 2000, 5);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Very high complexity should result in Sporadic classification
        assert_eq!(repo.group_family, "Sporadic");
        assert_eq!(repo.group_rank, 0);
        assert_eq!(repo.group_order, "7920");
    }

    #[test]
    fn test_group_classification_fork() {
        let mut repo = create_test_repo("forked_repo", 100, 50, 20, "Go", 50, 20);
        repo.is_fork = true;
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Forks should be classified as Cyclic
        assert_eq!(repo.group_family, "Cyclic");
        assert_eq!(repo.group_order, "2");
        assert_eq!(repo.group_rank, 1);
    }

    #[test]
    fn test_prime_order_calculation() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        let prime_order = repo.get_prime_order();
        
        // Should return a valid prime number
        assert!(prime_order > 1);
        assert!(is_prime(prime_order));
    }

    #[test]
    fn test_alternating_order_calculation() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        // Test A5 order: 5! / 2 = 60
        let order_a5 = repo.alternating_order(5);
        assert_eq!(order_a5, 60);
        
        // Test A6 order: 6! / 2 = 360
        let order_a6 = repo.alternating_order(6);
        assert_eq!(order_a6, 360);
    }

    #[test]
    fn test_lie_type_order_calculation() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        // Test PSL(2,2) order
        let order = repo.lie_type_order(2);
        assert!(order > 0);
        
        // Test PSL(3,2) order
        let order = repo.lie_type_order(3);
        assert!(order > 0);
    }

    #[test]
    fn test_tile_size_calculation() {
        let mut repo = create_test_repo("size_test", 1000, 500, 100, "Rust", 100, 10);
        repo.classify_group_family();
        
        let tile_size = repo.get_tile_size();
        
        // Tile size should be reasonable (between 50 and 150)
        assert!(tile_size >= 50.0);
        assert!(tile_size <= 150.0);
    }

    #[test]
    fn test_tile_color_cyclic() {
        let mut repo = create_test_repo("cyclic_color", 5, 2, 1, "C", 10, 100);
        repo.classify_group_family();
        
        assert_eq!(repo.get_tile_color(), "#FF6B6B"); // Red for Cyclic
    }

    #[test]
    fn test_tile_color_alternating() {
        let mut repo = create_test_repo("alternating_color", 1000, 500, 100, "C++", 100, 50);
        repo.classify_group_family();
        
        assert_eq!(repo.get_tile_color(), "#4ECDC4"); // Teal for Alternating
    }

    #[test]
    fn test_tile_color_lie_type() {
        let mut repo = create_test_repo("lie_type_color", 100000, 50000, 1000, "Python", 1000, 10);
        repo.classify_group_family();
        
        assert_eq!(repo.get_tile_color(), "#45B7D1"); // Blue for Lie Type
    }

    #[test]
    fn test_tile_color_sporadic() {
        let mut repo = create_test_repo("sporadic_color", 200000, 100000, 2000, "Java", 2000, 5);
        repo.classify_group_family();
        
        assert_eq!(repo.get_tile_color(), "#96CEB4"); // Green for Sporadic
    }

    #[test]
    fn test_group_theory_description() {
        let mut repo = create_test_repo("desc_test", 1000, 500, 100, "Rust", 100, 10);
        repo.classify_group_family();
        
        let description = repo.get_group_theory_description();
        
        assert!(description.contains(repo.group_family.as_str()));
        assert!(description.contains(&repo.group_order));
        assert!(!description.is_empty());
    }

    #[test]
    fn test_view_creation() {
        let view = RepositoryView::new("Test View".to_string(), "Test description".to_string());
        
        assert_eq!(view.name, "Test View");
        assert_eq!(view.description, "Test description");
        assert!(view.repositories.is_empty());
    }

    #[test]
    fn test_view_family_distribution_empty() {
        let view = RepositoryView::new("Empty View".to_string(), "Empty".to_string());
        let distribution = view.get_family_distribution();
        
        assert!(distribution.is_empty());
    }

    #[test]
    fn test_view_family_distribution_populated() {
        let mut view = RepositoryView::new("Populated View".to_string(), "Populated".to_string());
        
        let mut repo1 = create_test_repo("repo1", 100, 50, 10, "Rust", 100, 10);
        repo1.classify_group_family();
        
        let mut repo2 = create_test_repo("repo2", 200, 100, 20, "Python", 200, 5);
        repo2.classify_group_family();
        
        view.repositories.push(repo1);
        view.repositories.push(repo2);
        
        let distribution = view.get_family_distribution();
        
        assert_eq!(distribution.len(), 2);
        assert!(distribution.contains_key("Lie Type"));
        assert!(distribution.contains_key("Sporadic"));
    }

    #[test]
    fn test_view_complexity_stats_empty() {
        let view = RepositoryView::new("Empty View".to_string(), "Empty".to_string());
        let (min, max, avg) = view.get_complexity_stats();
        
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_view_complexity_stats_populated() {
        let mut view = RepositoryView::new("Populated View".to_string(), "Populated".to_string());
        
        let mut repo1 = create_test_repo("repo1", 100, 50, 10, "Rust", 100, 10);
        repo1.calculate_complexity();
        
        let mut repo2 = create_test_repo("repo2", 200, 100, 20, "Python", 200, 5);
        repo2.calculate_complexity();
        
        view.repositories.push(repo1);
        view.repositories.push(repo2);
        
        let (min, max, avg) = view.get_complexity_stats();
        
        assert!(min > 0.0);
        assert!(max > 0.0);
        assert!(avg > 0.0);
        assert!(min <= max);
    }

    #[test]
    fn test_atlas_data_generation() {
        let mut view = RepositoryView::new("Test Atlas".to_string(), "Test atlas description".to_string());
        
        let mut repo = create_test_repo("test_repo", 1000, 500, 100, "Rust", 100, 10);
        repo.classify_group_family();
        
        view.repositories.push(repo);
        
        let atlas_data = view.generate_atlas_data();
        
        assert!(atlas_data.contains("Test Atlas"));
        assert!(atlas_data.contains("Test atlas description"));
        assert!(atlas_data.contains("test_repo"));
        assert!(atlas_data.contains("Lie Type"));
        assert!(atlas_data.contains("512000"));
    }

    #[test]
    fn test_atlas_composer_creation() {
        let composer = RepositoryAtlasComposer::new();
        
        assert!(composer.views.is_empty());
    }

    #[test]
    fn test_atlas_composer_create_sample_repositories() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        assert!(!composer.views.is_empty());
        assert!(composer.views.contains_key("all_repos"));
        
        if let Some(all_repos) = composer.views.get("all_repos") {
            assert!(!all_repos.repositories.is_empty());
            assert_eq!(all_repos.repositories.len(), 5);
        }
    }

    #[test]
    fn test_atlas_composer_create_view() {
        let mut composer = RepositoryAtlasComposer::new();
        let view = composer.create_view("Test View".to_string(), "Test description".to_string());
        
        assert_eq!(view.name, "Test View");
        assert_eq!(view.description, "Test description");
        assert!(composer.views.contains_key("Test View"));
    }

    #[test]
    fn test_atlas_composer_generate_composition_json() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        match composer.generate_composition_json("all_repos".to_string()) {
            Ok(json_content) => {
                assert!(json_content.contains("All Repositories"));
                assert!(json_content.contains("Complete mathematical analysis"));
                assert!(json_content.contains("Lie Type"));
                assert!(json_content.contains("Sporadic"));
            }
            Err(e) => panic!("Failed to generate composition JSON: {}", e),
        }
    }

    #[test]
    fn test_atlas_composer_generate_composition_json_not_found() {
        let mut composer = RepositoryAtlasComposer::new();
        
        match composer.generate_composition_json("nonexistent".to_string()) {
            Err(e) => assert_eq!(e, "View not found"),
            Ok(_) => panic!("Should have failed for nonexistent view"),
        }
    }

    // Helper function to check if a number is prime
    fn is_prime(n: usize) -> bool {
        if n <= 1 {
            return false;
        }
        if n <= 3 {
            return true;
        }
        if n % 2 == 0 || n % 3 == 0 {
            return false;
        }
        for i in (5..=(n as f64).sqrt() as usize).step_by(6) {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
        }
        true
    }

    // Integration test for the complete workflow
    #[test]
    fn test_complete_workflow() {
        let mut composer = RepositoryAtlasComposer::new();
        
        // Create sample repositories
        composer.create_sample_repositories();
        
        // Verify all repos view was created
        assert!(composer.views.contains_key("all_repos"));
        
        // Create specialized views
        let cyclic_view = composer.create_view("Cyclic".to_string(), "Cyclic repos".to_string());
        let js_view = composer.create_view("JavaScript".to_string(), "JavaScript repos".to_string());
        
        // Generate compositions
        let cyclic_composition = composer.generate_composition_json("Cyclic".to_string());
        let js_composition = composer.generate_composition_json("JavaScript".to_string());
        
        assert!(cyclic_composition.is_ok());
        assert!(js_composition.is_ok());
        
        // Verify compositions contain expected data
        let cyclic_content = cyclic_composition.unwrap();
        let js_content = js_composition.unwrap();
        
        assert!(cyclic_content.contains("Cyclic"));
        assert!(js_content.contains("JavaScript"));
    }

    // Test edge cases
    #[test]
    fn test_edge_case_maximum_values() {
        let mut repo = create_test_repo("max_values", usize::MAX, usize::MAX, usize::MAX, "Rust", usize::MAX, 1);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Should not panic and should classify correctly
        assert!(!repo.group_family.is_empty());
        assert!(repo.complexity_score > 0.0);
    }

    #[test]
    fn test_edge_case_zero_values() {
        let mut repo = create_test_repo("zero_values", 0, 0, 0, "Rust", 0, 0);
        repo.calculate_complexity();
        repo.classify_group_family();
        
        // Should not panic and should classify correctly
        assert!(!repo.group_family.is_empty());
        assert!(repo.complexity_score >= 0.0);
    }

    // Test mathematical consistency
    #[test]
    fn test_mathematical_consistency() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let all_repos = composer.views.get("all_repos").unwrap();
        
        // Calculate total complexity
        let total_complexity: f64 = all_repos.repositories.iter().map(|r| r.complexity_score).sum();
        assert!(total_complexity > 0.0);
        
        // Verify all repositories have valid classifications
        for repo in &all_repos.repositories {
            assert!(!repo.group_family.is_empty());
            assert!(!repo.group_order.is_empty());
            assert!(!repo.name.is_empty());
            assert!(!repo.path.is_empty());
        }
    }

    // Test performance with large dataset
    #[test]
    fn test_large_dataset_performance() {
        let mut composer = RepositoryAtlasComposer::new();
        
        // Create a larger dataset
        let mut large_repos = Vec::new();
        for i in 0..100 {
            let mut repo = create_test_repo(&format!("repo_{}", i), i * 1000, i * 500, i * 100, "Rust", i * 10, i);
            repo.calculate_complexity();
            repo.classify_group_family();
            large_repos.push(repo);
        }
        
        let mut large_view = RepositoryView::new("Large Dataset".to_string(), "Performance test".to_string());
        large_view.repositories = large_repos;
        
        // Test that operations complete in reasonable time
        let start = std::time::Instant::now();
        let distribution = large_view.get_family_distribution();
        let complexity_stats = large_view.get_complexity_stats();
        let atlas_data = large_view.generate_atlas_data();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 1000); // Should complete within 1 second
        assert!(!distribution.is_empty());
        assert!(complexity_stats.0 > 0.0);
        assert!(!atlas_data.is_empty());
    }
}

// Unit tests for individual mathematical functions
#[cfg(test)]
mod mathematical_tests {
    use super::*;

    #[test]
    fn test_prime_number_properties() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        for (i, expected_prime) in primes.iter().enumerate() {
            let repo_copy = Repository::new("test".to_string(), "/test".to_string());
            // Simulate different complexity levels
            let mut repo_mutable = repo_copy;
            repo_mutable.complexity_score = i as f64 * 0.1;
            
            let prime_order = repo_mutable.get_prime_order();
            assert_eq!(prime_order, *expected_prime);
        }
    }

    #[test]
    fn test_alternating_group_orders() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        // Test known alternating group orders
        assert_eq!(repo.alternating_order(5), 60);   // |A5| = 5!/2 = 60
        assert_eq!(repo.alternating_order(6), 360);  // |A6| = 6!/2 = 360
        assert_eq!(repo.alternating_order(7), 2520); // |A7| = 7!/2 = 2520
    }

    #[test]
    fn test_lie_type_group_orders() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        // Test Lie type group orders (simplified calculation)
        let order_psl2_2 = repo.lie_type_order(2);
        let order_psl3_2 = repo.lie_type_order(3);
        
        assert!(order_psl2_2 > 0);
        assert!(order_psl3_2 > order_psl2_2);
        
        // PSL(2,2) should have order 6
        assert_eq!(order_psl2_2, 6);
    }

    #[test]
    fn test_group_family_transitions() {
        let repo = Repository::new("test".to_string(), "/test".to_string());
        
        // Test that complexity thresholds work correctly
        let test_cases = vec![
            (0.5, "Cyclic"),    // Low complexity
            (1.5, "Alternating"), // Medium complexity
            (2.5, "Lie Type"),   // High complexity
            (3.5, "Sporadic"),   // Very high complexity
        ];
        
        for (complexity, expected_family) in test_cases {
            let mut repo_test = repo.clone();
            repo_test.complexity_score = complexity;
            repo_test.classify_group_family();
            
            assert_eq!(repo_test.group_family, expected_family);
        }
    }
}

// Integration tests for file I/O operations
#[cfg(test)]
mod file_io_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_atlas_data_file_generation() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let test_output_path = "./test_output.md";
        
        // Export atlas data to file
        if let Some(all_repos) = composer.views.get("all_repos") {
            let atlas_content = all_repos.generate_atlas_data();
            fs::write(test_output_path, atlas_content).unwrap();
            
            // Verify file was created and contains expected content
            assert!(Path::new(test_output_path).exists());
            
            let file_content = fs::read_to_string(test_output_path).unwrap();
            assert!(file_content.contains("Repository Mathematical Atlas"));
            assert!(file_content.contains("All Repositories"));
            
            // Clean up
            fs::remove_file(test_output_path).unwrap();
        }
    }

    #[test]
    fn test_json_composition_file_generation() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let test_output_path = "./test_composition.json";
        
        // Generate JSON composition
        match composer.generate_composition_json("all_repos".to_string()) {
            Ok(json_content) => {
                fs::write(test_output_path, json_content).unwrap();
                
                // Verify file was created and contains expected content
                assert!(Path::new(test_output_path).exists());
                
                let file_content = fs::read_to_string(test_output_path).unwrap();
                assert!(file_content.contains("All Repositories"));
                assert!(file_content.contains("Lie Type"));
                
                // Clean up
                fs::remove_file(test_output_path).unwrap();
            }
            Err(e) => panic!("Failed to generate JSON composition: {}", e),
        }
    }
}

// Benchmark tests for performance measurement
#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_complexity_calculation() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let start = Instant::now();
        
        // Calculate complexity for all repositories
        for repo in &mut composer.views["all_repos"].repositories {
            repo.calculate_complexity();
        }
        
        let duration = start.elapsed();
        println!("Complexity calculation for 5 repos: {:?}", duration);
        
        assert!(duration.as_millis() < 100); // Should be very fast
    }

    #[test]
    fn benchmark_classification() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let start = Instant::now();
        
        // Classify all repositories
        for repo in &mut composer.views["all_repos"].repositories {
            repo.classify_group_family();
        }
        
        let duration = start.elapsed();
        println!("Classification for 5 repos: {:?}", duration);
        
        assert!(duration.as_millis() < 100); // Should be very fast
    }

    #[test]
    fn benchmark_atlas_generation() {
        let mut composer = RepositoryAtlasComposer::new();
        composer.create_sample_repositories();
        
        let start = Instant::now();
        
        // Generate atlas data
        let atlas_data = composer.views["all_repos"].generate_atlas_data();
        
        let duration = start.elapsed();
        println!("Atlas generation: {:?}", duration);
        
        assert!(!atlas_data.is_empty());
        assert!(duration.as_millis() < 100); // Should be very fast
    }
}