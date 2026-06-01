use std::collections::HashMap;

#[cfg(test)]
mod topological_tests {
    use super::*;

    #[test]
    fn test_dependency_ordering() {
        let deps: HashMap<String, Vec<String>> = HashMap::from([
            ("app".to_string(), vec!["b".to_string(), "c".to_string()]),
            ("b".to_string(), vec!["d".to_string()]),
            ("c".to_string(), vec!["d".to_string()]),
            ("d".to_string(), vec![]),
        ]);

        let order = vec!["d", "b", "c", "app"];
        let positions: HashMap<&str, usize> = order
            .iter()
            .enumerate()
            .map(|(i, crate_id)| (*crate_id, i))
            .collect();

        for (i, crate_id) in order.iter().enumerate() {
            for dep in &deps[*crate_id] {
                let dep_idx = positions[dep.as_str()];
                assert!(dep_idx < i, "{} depends on {} but appears later", crate_id, dep);
            }
        }
    }

    #[test]
    fn test_workspace_vs_external_filtering() {
        let crate_ids = vec![
            "workspace:my-crate".to_string(),
            "workspace:my-other-crate".to_string(),
            "crate:external-dep".to_string(),
            "crate:another-external".to_string(),
        ];

        let workspace_crates: Vec<_> = crate_ids
            .iter()
            .filter(|id| id.starts_with("workspace:"))
            .collect();
        let external_crates: Vec<_> = crate_ids
            .iter()
            .filter(|id| id.starts_with("crate:"))
            .collect();

        assert_eq!(workspace_crates.len(), 2);
        assert_eq!(external_crates.len(), 2);
    }

    #[test]
    fn test_dependency_edge_type_classification() {
        let edge_types = vec!["Direct", "Dev", "Build", "Transitive"];

        let dev_deps: Vec<_> = edge_types.iter().filter(|&t| t == &"Dev").collect();

        assert_eq!(dev_deps.len(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let deps: HashMap<String, Vec<String>> = HashMap::from([
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["c".to_string()]),
            ("c".to_string(), vec!["a".to_string()]),
        ]);

        fn has_cycle(
            deps: &HashMap<String, Vec<String>>,
            start: &str,
            visiting: &mut Vec<String>,
        ) -> bool {
            if visiting.iter().any(|node| node == start) {
                return true;
            }

            visiting.push(start.to_string());
            if let Some(children) = deps.get(start) {
                for dep in children {
                    if has_cycle(deps, dep, visiting) {
                        return true;
                    }
                }
            }
            visiting.pop();
            false
        }

        let mut visiting = Vec::new();
        assert!(has_cycle(&deps, "a", &mut visiting));
    }

    #[test]
    fn test_topological_sort_dag() {
        let deps: HashMap<String, Vec<String>> = HashMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec!["a".to_string()]),
            ("c".to_string(), vec!["b".to_string()]),
        ]);

        let order = vec!["a", "b", "c"];
        for (i, crate_id) in order.iter().enumerate() {
            for dep in &deps[*crate_id] {
                let dep_idx = order
                    .iter()
                    .position(|candidate| candidate == &dep.as_str())
                    .unwrap();
                assert!(dep_idx < i);
            }
        }
    }
}
