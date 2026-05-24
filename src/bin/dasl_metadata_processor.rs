//! DASL Metadata Processor CLI
//!
//! Processes ~/dasl/git.txt to extract metadata from .gitmodules files
//! across all DASL projects for QA tracking.

use cargo_vendormod::dasl_metadata_processor::DaslMetadata;

fn main() -> anyhow::Result<()> {
    println!("📊 DASL Metadata Processor");
    println!("=========================");

    let metadata = DaslMetadata::process_git_txt("/home/mdupont/dasl/git.txt")?;

    println!("\n📈 Statistics:");
    println!("   Total projects: {}", metadata.total_projects);
    println!("   Total submodules: {}", metadata.total_submodules);

    println!("\n📦 By Type:");
    for (type_name, entries) in &metadata.by_type {
        println!("   {}: {}", type_name, entries.len());
    }

    println!("\n🔧 By Fuzzer Engine:");
    for (engine, entries) in &metadata.by_fuzzer_engine {
        println!("   {}: {}", engine, entries.len());
        for entry in entries {
            println!("      - {} ({})", entry.path, entry.url);
        }
    }

    println!("\n🎯 Fuzzer Harnesses: {}", metadata.fuzzer_harnesses().len());
    println!("\n📋 JSON Output:");
    println!("{}", metadata.to_json());

    Ok(())
}