use anyhow::Result;
use crate::global_dep_graph::GlobalDependencyGraphBuilder;
use clap::Parser;
use std::path::PathBuf;
use ipld_car_ipc_shmem_linux::car_store::CarStore;
use hex;
use crate::nora_index;

#[derive(Parser, Debug, Clone)]
pub struct NoraIndexArgs {
    /// Path to workspace root directory
    #[arg(long)]
    pub workspace_path: PathBuf,

    /// Shmem server socket path (default: @ipld_car_shmem)
    #[arg(long, default_value = "@ipld_car_shmem")]
    pub shmem_socket: String,

    /// Cache directory for CAR pages (default: /mnt/data1/dasl-cache)
    #[arg(long, default_value = "/mnt/data1/dasl-cache")]
    pub cache_dir: PathBuf,

    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

pub fn run_nora_index(args: &NoraIndexArgs) -> Result<()> {
    println!("═══ Nora Index ═══");
    println!("Workspace: {}", args.workspace_path.display());
    println!("Cache dir: {}", args.cache_dir.display());
    println!();

    let mut builder = GlobalDependencyGraphBuilder::new(args.workspace_path.clone());
    builder.set_options(true, true, true);
    let graph = builder.build_global_graph()?;

    println!("Found {} crates in dependency graph", graph.nodes.len());

    let mut store = CarStore::open(&args.cache_dir, 36 * 1024 * 1024 * 1024)?;
    println!("Opened CarStore");

    for node in &graph.nodes {
        if args.verbose {
            println!("Indexing: {}", node.crate_name);
        }

        let local = nora_index::create_local_mirror(
            &node.crate_name,
            &node.version,
            None,
        );
        let local_bytes = nora_index::encode_as_dag_cbor(&local)?;
        let local_car = nora_index::wrap_as_car(&local_bytes)?;
        let local_cid = store.put_block(&format!("nora/local/{}", node.crate_name), &format!("Nora local mirror info for {}", node.crate_name), false, &local_car)?;
        let local_cid_bytes = hex::decode(&local_cid)?;
        let mut local_cid_hash = [0u8; 32];
        local_cid_hash.copy_from_slice(&local_cid_bytes[4..]);

        let nix = nora_index::create_nix_build(
            &node.crate_name,
            &format!("/mnt/data1/nix/time/2024/05/28/depot/tvix/vendored/output/flakes/{}/flake.nix", node.crate_name),
        );
        let nix_bytes = nora_index::encode_as_dag_cbor(&nix)?;
        let nix_car = nora_index::wrap_as_car(&nix_bytes)?;
        let nix_cid = store.put_block(&format!("nora/nix/{}", node.crate_name), &format!("Nora Nix build info for {}", node.crate_name), false, &nix_car)?;
        let nix_cid_bytes = hex::decode(&nix_cid)?;
        let mut nix_cid_hash = [0u8; 32];
        nix_cid_hash.copy_from_slice(&nix_cid_bytes[4..]);

        let pipelight = nora_index::create_pipelight_info(&node.crate_name);
        let pipelight_bytes = nora_index::encode_as_dag_cbor(&pipelight)?;
        let pipelight_car = nora_index::wrap_as_car(&pipelight_bytes)?;
        let pipelight_cid = store.put_block(&format!("nora/pipelight/{}", node.crate_name), &format!("Nora Pipelight info for {}", node.crate_name), false, &pipelight_car)?;
        let pipelight_cid_bytes = hex::decode(&pipelight_cid)?;
        let mut pipelight_cid_hash = [0u8; 32];
        pipelight_cid_hash.copy_from_slice(&pipelight_cid_bytes[4..]);

        let vendor = nora_index::create_vendorsization(
            &node.crate_name,
            &format!("/mnt/data1/time-2024/05/28/depot/tvix/vendor/"),
        );
        let vendor_bytes = nora_index::encode_as_dag_cbor(&vendor)?;
        let vendor_car = nora_index::wrap_as_car(&vendor_bytes)?;
        let vendor_cid = store.put_block(&format!("nora/vendor/{}", node.crate_name), &format!("Nora vendorsization info for {}", node.crate_name), false, &vendor_car)?;
        let vendor_cid_bytes = hex::decode(&vendor_cid)?;
        let mut vendor_cid_hash = [0u8; 32];
        vendor_cid_hash.copy_from_slice(&vendor_cid_bytes[4..]);

        let patches = nora_index::create_patches_info();
        let patches_bytes = nora_index::encode_as_dag_cbor(&patches)?;
        let patches_car = nora_index::wrap_as_car(&patches_bytes)?;
        let patches_cid = store.put_block(&format!("nora/patches/{}", node.crate_name), &format!("Nora patches info for {}", node.crate_name), false, &patches_car)?;
        let patches_cid_bytes = hex::decode(&patches_cid)?;
        let mut patches_cid_hash = [0u8; 32];
        patches_cid_hash.copy_from_slice(&patches_cid_bytes[4..]);

        let consumers = nora_index::create_consumers_info(
            &node.crate_name,
            &args.workspace_path.display().to_string(),
        );
        let consumers_bytes = nora_index::encode_as_dag_cbor(&consumers)?;
        let consumers_car = nora_index::wrap_as_car(&consumers_bytes)?;
        let consumers_cid = store.put_block(&format!("nora/consumers/{}", node.crate_name), &format!("Nora consumers info for {}", node.crate_name), false, &consumers_car)?;
        let consumers_cid_bytes = hex::decode(&consumers_cid)?;
        let mut consumers_cid_hash = [0u8; 32];
        consumers_cid_hash.copy_from_slice(&consumers_cid_bytes[4..]);

        let (_entry, index_car) = nora_index::create_crate_index(
            &node.crate_name,
            &local_cid_hash,
            &nix_cid_hash,
            &pipelight_cid_hash,
            &vendor_cid_hash,
            &patches_cid_hash,
            &consumers_cid_hash,
        );

        let index_cid = store.put_block(&format!("nora/index/{}", node.crate_name), &format!("Nora index for {}", node.crate_name), false, &index_car)?;
        
        // Also store in block index for name-based lookup
        let index_path = format!("crates/{}/index", node.crate_name);
        let _ = store.put_block(&index_path, "Nora crate index entry", true, &nora_index::encode_as_dag_cbor(&_entry)?);

        if args.verbose {
            println!("  local CID: {}", hex::encode(&local_cid[..8]));
            println!("  index CID: {}", hex::encode(&index_cid[..8]));
        }
    }

    let stats = store.stats();
    println!("
Index complete.");
    println!("  Total crates indexed: {}", graph.nodes.len());

    Ok(())
}
