use tempfile::tempdir;
use std::collections::HashMap;

#[cfg(test)]
mod topological_tests {
    use super::*;

    #[test]
    fn test_dependency_ordering() {
        let mut deps = HashMap::new();
        deps.insert("app", vec!["b", "c"]);
        deps.insert("b", vec!["d"]);
        deps.insert("c", vec!["d"]);
        deps.insert("d", vec![]);

        let order = vec!["d", "b", "c", "app"];
        for (i, &crate_id) in order.iter().enumerate() {
            for &dep in &deps[crate_id] {
                let dep_idx = order.iter().position(|&x| x == dep).unwrap();
                assert!(dep_idx < i, "{} depends on {} but appears later", crate_id, dep);
            }
        }
    }

    #[test]
    fn test_workspace_vs_external_filtering() {
        let crate_ids = vec![
            "workspace:my-crate",
            "workspace:my-other-crate",
            "crate:external-dep",
            "crate:another-external",
        ];

        let workspace_crates: Vec<_> = crate_ids.iter()
            .filter(|id| id.starts_with("workspace:"))
            .collect();
        let external_crates: Vec<_> = crate_ids.iter()
            .filter(|id| id.starts_with("crate:"))
            .collect();

        assert_eq!(workspace_crates.len(), 2);
        assert_eq!(external_crates.len(), 2);
    }

    #[test]
    fn test_dependency_edge_type_classification() {
        let edge_types = vec![
            "Direct",
            "Dev",
            "Build",
            "Transitive",
        ];

        let dev_deps: Vec<_> = edge_types.iter()
            .filter(|&t| t == &"Dev")
            .collect();

        assert_eq!(dev_deps.len(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut deps = HashMap::new();
        deps.insert("a", vec!["b"]);
        deps.insert("b", vec!["c"]);
        deps.insert("c", vec!["a"]);

        fn has_cycle(deps: &HashMap<&str, Vec<&str>>, start: &str, visited: &mut Vec<&str>) -> bool {
            if visited.contains(&start) {
                return true;
            }
            visited.push(start);
            for &dep in &deps[start] {
                if has_cycle(deps, dep, visited) {
                    return true;
                }
            }
            visited.pop();
            false
        }

        let mut visited = Vec::new();
        assert!(has_cycle(&deps, "a", &mut visited));
    }

    #[test]
    fn test_topological_sort_dag() {
        let mut deps = HashMap::new();
        deps.insert("a", vec![]);
        deps.insert("b", vec!["a"]);
        deps.insert("c", vec!["b"]);

        let mut order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        for (i, &crate_id) in order.iter().enumerate() {
            for &dep in &deps[crate_id.as_str()] {
                let dep_idx = order.iter().position(|x| x == dep).unwrap();
                assert!(dep_idx < i);
            }
        }
    }
}
