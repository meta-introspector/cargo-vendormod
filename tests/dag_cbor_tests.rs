use cargo_vendormod::flake_check::{TargetProject, ProjectCategory, target_projects, check_flake_coverage};
use tempfile::tempdir;
use std::path::PathBuf;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_project_construction() {
        let proj = TargetProject {
            name: "serde_ipld_dagcbor".into(),
            category: ProjectCategory::CborLib,
            source: "ipld/serde_ipld_dagcbor".into(),
            flake_dir: "serde_ipld_dagcbor".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        };
        assert_eq!(proj.category, ProjectCategory::CborLib);
        assert_eq!(proj.source, "ipld/serde_ipld_dagcbor");
        assert!(!proj.has_flake_nix);
    }

    #[test]
    fn test_project_category_display() {
        assert_eq!(ProjectCategory::CborLib.to_string(), "CBOR Lib");
        assert_eq!(ProjectCategory::MainProject.to_string(), "Main Project");
        assert_eq!(ProjectCategory::FuzzTool.to_string(), "Fuzz Tool");
        assert_eq!(ProjectCategory::TestSuite.to_string(), "Test Suite");
    }

    #[test]
    fn test_target_projects_includes_ipld_crates() {
        let projects = target_projects();
        let names: Vec<_> = projects.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"serde_ipld_dagcbor".to_string()));
        assert!(names.contains(&"ipld-core".to_string()));
        assert!(names.contains(&"libipld (w/ fuzz)".to_string()));
        assert!(names.contains(&"ipld-dagpb".to_string()));
        assert!(names.contains(&"serde_cbor (pyfisch)".to_string()));
        assert!(names.contains(&"dasl".to_string()));
    }

    #[test]
    fn test_target_projects_count() {
        let projects = target_projects();
        assert!(!projects.is_empty());
        assert!(projects.len() >= 6, "Expected >= 6 projects, got {}", projects.len());
    }

    #[test]
    fn test_project_category_equality() {
        assert_eq!(ProjectCategory::CborLib, ProjectCategory::CborLib);
        assert_ne!(ProjectCategory::CborLib, ProjectCategory::MainProject);
    }

    #[test]
    fn test_flake_check_with_empty_dir() {
        let dir = tempdir().unwrap();
        let flakes_dir = dir.path().join("flakes");
        fs::create_dir_all(&flakes_dir).unwrap();

        let report = check_flake_coverage(&flakes_dir);
        assert!(report.is_ok());
        let cov = report.unwrap();
        assert_eq!(cov.summary.total, target_projects().len());
        assert_eq!(cov.summary.complete, 0);
        assert_eq!(cov.summary.partial, 0);
        assert_eq!(cov.summary.missing, cov.summary.total);
    }

    #[test]
    fn test_flake_check_with_real_subdir() {
        let dir = tempdir().unwrap();
        let flakes_dir = dir.path().join("flakes");
        fs::create_dir_all(flakes_dir.join("serde_ipld_dagcbor")).unwrap();
        fs::write(flakes_dir.join("serde_ipld_dagcbor").join("flake.nix"), "").unwrap();

        let report = check_flake_coverage(&flakes_dir);
        assert!(report.is_ok());
    }

    #[test]
    fn test_cbor_lib_projects_have_correct_source_prefix() {
        let projects = target_projects();
        let cbor: Vec<_> = projects
            .iter()
            .filter(|p| p.category == ProjectCategory::CborLib)
            .collect();

        for proj in cbor {
            assert!(proj.source.contains('/'), "Expected org/repo source for {}", proj.name);
        }
    }
}
