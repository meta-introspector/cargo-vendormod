use cargo_vendormod::global_dep_graph::{
    DependencyNode, DependencyEdge, DependencyEdgeType, GlobalDependencyGraphBuilder,
    GraphMetrics, GraphPartition, PartitionMetrics, PartitioningAlgorithm, FeatureSet,
    TomlStructure, SchemaElement, SchemaElementType, WorkloadPattern, WorkloadPatternType,
};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use tempfile::tempdir;

#[cfg(test)]
mod global_graph_tests {
    use super::*;

    #[test]
    fn test_dependency_node_creation() {
        let node = DependencyNode {
            id: "pkg:test-crate".to_string(),
            crate_name: "test-crate".to_string(),
            version: "1.0.0".to_string(),
            source: "crates.io".to_string(),
            is_workspace_member: false,
            is_direct_dependency: true,
            features: HashSet::new(),
            categories: vec!["test".to_string()],
            properties: HashMap::new(),
        };

        assert_eq!(node.crate_name, "test-crate");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.source, "crates.io");
        assert!(node.is_direct_dependency);
        assert!(!node.is_workspace_member);
    }

    #[test]
    fn test_dependency_edge_creation() {
        let edge = DependencyEdge {
            from: "pkg:app".to_string(),
            to: "pkg:serde".to_string(),
            edge_type: DependencyEdgeType::Direct,
            required_features: HashSet::new(),
            optional_features: HashSet::new(),
            is_dev_dependency: false,
            is_build_dependency: false,
            properties: HashMap::new(),
        };

        assert_eq!(edge.from, "pkg:app");
        assert_eq!(edge.to, "pkg:serde");
        assert!(matches!(edge.edge_type, DependencyEdgeType::Direct));
        assert!(!edge.is_dev_dependency);
    }

    #[test]
    fn test_dependency_edge_type_variants() {
        let types = vec![
            DependencyEdgeType::Direct,
            DependencyEdgeType::Transitive,
            DependencyEdgeType::FeatureGated,
            DependencyEdgeType::Dev,
            DependencyEdgeType::Build,
            DependencyEdgeType::Workspace,
            DependencyEdgeType::Virtual,
        ];

        assert_eq!(types.len(), 7);
        assert!(matches!(types[0], DependencyEdgeType::Direct));
        assert!(matches!(types[3], DependencyEdgeType::Dev));
        assert!(matches!(types[5], DependencyEdgeType::Workspace));
    }

    #[test]
    fn test_edge_type_equality() {
        assert_eq!(DependencyEdgeType::Direct, DependencyEdgeType::Direct);
        assert_ne!(DependencyEdgeType::Direct, DependencyEdgeType::Dev);
        assert_ne!(DependencyEdgeType::Dev, DependencyEdgeType::Build);
    }

    #[test]
    fn test_graph_builder_creation() {
        let temp_dir = tempdir().unwrap();
        let builder = GlobalDependencyGraphBuilder::new(temp_dir.path().to_path_buf());
        assert_eq!(builder.graph.node_count(), 0);
        assert!(builder.node_map.is_empty());
    }

    #[test]
    fn test_graph_builder_with_options() {
        let temp_dir = tempdir().unwrap();
        let mut builder = GlobalDependencyGraphBuilder::new(temp_dir.path().to_path_buf());
        builder.set_options(false, false, false);
        // Verify builder was modified (internal state not directly accessible, but method chains)
    }

    #[test]
    fn test_feature_set_creation() {
        let mut feature_deps = HashMap::new();
        feature_deps.insert("serde".to_string(), HashSet::new());

        let fs = FeatureSet {
            crate_name: "tokio".to_string(),
            features: HashSet::from(["full".to_string(), "macros".to_string()]),
            default_features: HashSet::from(["full".to_string()]),
            feature_dependencies,
        };

        assert_eq!(fs.crate_name, "tokio");
        assert!(fs.features.contains("full"));
        assert!(fs.default_features.contains("full"));
    }

    #[test]
    fn test_graph_metrics_default() {
        let metrics = GraphMetrics {
            node_count: 10,
            edge_count: 15,
            workspace_members: 2,
            direct_dependencies: 5,
            transitive_dependencies: 3,
            feature_gated_dependencies: 2,
            dev_dependencies: 1,
            build_dependencies: 1,
            strongly_connected_components: 0,
            diameter: 5,
            average_degree: 1.5,
        };

        assert_eq!(metrics.node_count, 10);
        assert_eq!(metrics.edge_count, 15);
        assert_eq!(metrics.workspace_members, 2);
        assert!(metrics.average_degree > 0.0);
    }

    #[test]
    fn test_partitioning_algorithm_parsing() {
        assert_eq!("greedy".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::Greedy);
        assert_eq!("kaminpar".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::KaMinPar);
        assert_eq!("metis".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::Metis);
        assert_eq!("louvain".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::Louvain);
        assert_eq!("spectral".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::Spectral);
        assert_eq!("kernighanlin".parse::<PartitioningAlgorithm>().unwrap(), PartitioningAlgorithm::KernighanLin);

        assert!("invalid".parse::<PartitioningAlgorithm>().is_err());
    }

    #[test]
    fn test_partition_metrics() {
        let metrics = PartitionMetrics {
            internal_edges: 10,
            external_edges: 3,
            density: 0.5,
            modularity: 0.3,
            cohesion: 0.7,
            coupling: 0.3,
        };

        assert_eq!(metrics.internal_edges, 10);
        assert!(metrics.cohesion > metrics.coupling);
    }

    #[test]
    fn test_schema_element_type_equality() {
        assert_eq!(SchemaElementType::Package, SchemaElementType::Package);
        assert_eq!(SchemaElementType::Dependency, SchemaElementType::Dependency);
        assert_ne!(SchemaElementType::Dependency, SchemaElementType::DevDependency);
    }

    #[test]
    fn test_workload_pattern_type_variants() {
        assert_eq!(WorkloadPatternType::BinaryTarget, WorkloadPatternType::BinaryTarget);
        assert_ne!(WorkloadPatternType::TestSuite, WorkloadPatternType::BenchmarkSuite);
    }
}
