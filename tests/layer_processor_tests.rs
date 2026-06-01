use cargo_vendormod::global_dep_graph::{GlobalDependencyGraph, GlobalDependencyGraphBuilder, DependencyNode, DependencyEdge, DependencyEdgeType};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use tempfile::tempdir;

#[cfg(test)]
mod layer_processor_tests {
    use super::*;

    #[test]
    fn test_graph_node_lookup() {
        let temp_dir = tempdir().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();

        let mut builder = GlobalDependencyGraphBuilder::new(temp_dir.path().to_path_buf());

        // Manually add nodes to simulate graph state
        let node_a = DependencyNode {
            id: "crate:serde".to_string(),
            crate_name: "serde".to_string(),
            version: "1.0.0".to_string(),
            source: "crates.io".to_string(),
            is_workspace_member: false,
            is_direct_dependency: true,
            features: HashSet::new(),
            categories: vec![],
            properties: HashMap::new(),
        };
        builder.node_map.insert(node_a.id.clone(), builder.graph.add_node(node_a));

        let found = builder.graph.node_weights()
            .find(|n| n.crate_name == "serde");

        assert!(found.is_some());
        assert_eq!(found.unwrap().version, "1.0.0");
    }

    #[test]
    fn test_edge_lookup_between_nodes() {
        let mut builder = GlobalDependencyGraphBuilder::new(PathBuf::from("."));

        let app = DependencyNode {
            id: "crate:app".to_string(),
            crate_name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: "workspace".to_string(),
            is_workspace_member: true,
            is_direct_dependency: true,
            features: HashSet::new(),
            categories: vec![],
            properties: HashMap::new(),
        };

        let serde = DependencyNode {
            id: "crate:serde".to_string(),
            crate_name: "serde".to_string(),
            version: "1.0.0".to_string(),
            source: "crates.io".to_string(),
            is_workspace_member: false,
            is_direct_dependency: false,
            features: HashSet::new(),
            categories: vec![],
            properties: HashMap::new(),
        };

        let app_idx = builder.graph.add_node(app);
        let serde_idx = builder.graph.add_node(serde);

        builder.graph.add_edge(app_idx, serde_idx, DependencyEdge {
            from: "crate:app".to_string(),
            to: "crate:serde".to_string(),
            edge_type: DependencyEdgeType::Direct,
            required_features: HashSet::new(),
            optional_features: HashSet::new(),
            is_dev_dependency: false,
            is_build_dependency: false,
            properties: HashMap::new(),
        });

        let edges: Vec<_> = builder.graph.edges(app_idx).collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target(), serde_idx);
        assert!(matches!(edges[0].weight().edge_type, DependencyEdgeType::Direct));
    }

    #[test]
    fn test_node_map_consistency() {
        let mut builder = GlobalDependencyGraphBuilder::new(PathBuf::from("."));
        let node = DependencyNode {
            id: "crate:test".to_string(),
            crate_name: "test".to_string(),
            version: "1.0.0".to_string(),
            source: "crates.io".to_string(),
            is_workspace_member: false,
            is_direct_dependency: false,
            features: HashSet::new(),
            categories: vec![],
            properties: HashMap::new(),
        };

        let idx = builder.graph.add_node(node.clone());
        builder.node_map.insert(node.id.clone(), idx);

        assert!(builder.node_map.contains_key(&node.id));
        assert_eq!(builder.node_map[&node.id], idx);
    }

    #[test]
    fn test_external_dependency_identification() {
        let nodes = vec![
            DependencyNode {
                id: "ws:app".to_string(),
                crate_name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: "workspace".to_string(),
                is_workspace_member: true,
                is_direct_dependency: true,
                features: HashSet::new(),
                categories: vec![],
                properties: HashMap::new(),
            },
            DependencyNode {
                id: "crate:serde".to_string(),
                crate_name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: "crates.io".to_string(),
                is_workspace_member: false,
                is_direct_dependency: false,
                features: HashSet::new(),
                categories: vec![],
                properties: HashMap::new(),
            },
        ];

        let external: Vec<_> = nodes.iter().filter(|n| !n.is_workspace_member).collect();
        let workspace: Vec<_> = nodes.iter().filter(|n| n.is_workspace_member).collect();

        assert_eq!(external.len(), 1);
        assert_eq!(workspace.len(), 1);
        assert_eq!(external[0].crate_name, "serde");
        assert_eq!(workspace[0].crate_name, "app");
    }
}
