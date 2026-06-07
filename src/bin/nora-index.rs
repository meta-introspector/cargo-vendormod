use anyhow::Result;
use clap::Parser;
use cargo_vendormod::nora_indexer::{run_nora_index, NoraIndexArgs};

fn main() -> Result<()> {
    let args = NoraIndexArgs::parse();
    run_nora_index(&args)
}
