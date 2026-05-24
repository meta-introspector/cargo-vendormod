use std::collections::HashMap;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

fn main() {
    // Build a simple dependency graph for our own dependencies
    let mut graph = Graph::<String, ()>::new();
    
    // Add nodes for our main dependencies
    let cargo_vendormod = graph.add_node("cargo-vendormod".to_string());
    let anyhow = graph.add_node("anyhow".to_string());
    let cargo_metadata = graph.add_node("cargo_metadata".to_string());
    let clap = graph.add_node("clap".to_string());
    let petgraph = graph.add_node("petgraph".to_string());
    let serde = graph.add_node("serde".to_string());
    let toml_edit = graph.add_node("toml_edit".to_string());
    
    // Add edges representing dependencies
    graph.add_edge(cargo_vendormod, anyhow, ());
    graph.add_edge(cargo_vendormod, cargo_metadata, ());
    graph.add_edge(cargo_vendormod, clap, ());
    graph.add_edge(cargo_vendormod, petgraph, ());
    graph.add_edge(cargo_vendormod, serde, ());
    graph.add_edge(cargo_vendormod, toml_edit, ());
    
    // Add some transitive dependencies
    let serde_core = graph.add_node("serde_core".to_string());
    let semver = graph.add_node("semver".to_string());
    let camino = graph.add_node("camino".to_string());
    
    graph.add_edge(cargo_metadata, serde_core, ());
    graph.add_edge(cargo_metadata, semver, ());
    graph.add_edge(cargo_metadata, camino, ());
    
    println!("Dependency graph built with {} nodes and {} edges", 
             graph.node_count(), graph.edge_count());
    
    // Print the graph structure
    for node in graph.node_indices() {
        let node_name = &graph[node];
        let deps: Vec<_> = graph.neighbors_directed(node, petgraph::Direction::Outgoing)
            .map(|n| graph[n].clone())
            .collect();
        println!("{}: {:?}", node_name, deps);
    }
    
    // Perform topological sort
    let topo_order = petgraph::algo::toposort(&graph, None);
    match topo_order {
        Ok(order) => {
            println!("\nTopological order:");
            for (i, node_idx) in order.iter().enumerate() {
                println!("{}: {}", i+1, graph[*node_idx]);
            }
        }
        Err(cycle) => {
            println!("Cycle detected involving node: {}", graph[cycle.node_id()]);
        }
    }
}