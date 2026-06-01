use cargo_vendormod::global_dep_graph::{
    DependencyNode, GlobalDependencyGraph, GraphMetrics,
};
use cargo_vendormod::layer_processor::LayerProcessor;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(test)]
mod layer_processor_tests {
    use super::*;

    fn empty_metrics() -> GraphMetrics {
        GraphMetrics {
            node_count: 0,
            edge_count: 0,
            workspace_members: 0,
            direct_dependencies: 0,
            transitive_dependencies: 0,
            feature_gated_dependencies: 0,
            dev_dependencies: 0,
            build_dependencies: 0,
            strongly_connected_components: 0,
            diameter: 0,
            average_degree: 0.0,
        }
    }

    fn dependency_node(id: &str, crate_name: &str, source: &str, is_workspace_member: bool) -> DependencyNode {
        DependencyNode {
            id: id.to_string(),
            crate_name: crate_name.to_string(),
            version: "0.1.0".to_string(),
            source: source.to_string(),
            is_workspace_member,
            is_direct_dependency: true,
            features: HashSet::new(),
            categories: vec![],
            properties: HashMap::new(),
        }
    }

    fn make_graph(
        workspace_path: PathBuf,
        nodes: Vec<DependencyNode>,
        publish_order: Vec<String>,
        external_dependency_order: Vec<String>,
    ) -> GlobalDependencyGraph {
        GlobalDependencyGraph {
            nodes,
            edges: Vec::new(),
            features: Vec::new(),
            toml_structures: Vec::new(),
            strongly_connected_components: Vec::new(),
            partitions: Vec::new(),
            metrics: empty_metrics(),
            workspace_path,
            publish_order,
            topological_order: Vec::new(),
            external_dependency_order,
        }
    }

    #[test]
    fn test_process_layer1_writes_flake_for_external_dependency() {
        let workspace_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();

        let node = dependency_node("crate:serde", "serde", "crates.io", false);
        let graph = make_graph(
            workspace_dir.path().to_path_buf(),
            vec![node],
            Vec::new(),
            vec!["crate:serde".to_string()],
        );

        let processor = LayerProcessor::new(
            graph,
            workspace_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            PathBuf::from("git"),
            home_dir.path().to_path_buf(),
        );

        processor.process_layer1().unwrap();

        assert!(output_dir.path().join("layer1/serde/flake.nix").exists());
        assert!(output_dir.path().join("crates-io-cache/serde").exists());
    }

    #[test]
    fn test_process_layer2_builds_workspace_member() {
        let workspace_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();

        let crate_dir = workspace_dir.path().join("app");
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        fs::write(crate_dir.join("src/lib.rs"), "pub fn app() -> u32 { 1 }\n").unwrap();

        let node = dependency_node("workspace:app", "app", "workspace", true);
        let graph = make_graph(
            workspace_dir.path().to_path_buf(),
            vec![node],
            vec!["workspace:app".to_string()],
            Vec::new(),
        );

        let processor = LayerProcessor::new(
            graph,
            workspace_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            PathBuf::from("git"),
            home_dir.path().to_path_buf(),
        );

        processor.process_layer2().unwrap();

        assert!(output_dir.path().join("layer2/app/flake.nix").exists());
    }
}
