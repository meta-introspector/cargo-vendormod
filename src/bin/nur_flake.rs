//! # NUR Flake Generator — CLI Binary
//!
//! Reads repos.json and repos.json.lock, generates a comprehensive flake.nix
//! listing all NUR repositories as flake inputs.
//!
//! Usage:
//!   cargo-vendormod nur-flake --repos-json /path/to/repos.json --lock-json /path/to/repos.json.lock --output /path/to/flake.nix
//!
//! Check mode (verify without overwriting):
//!   cargo-vendormod nur-flake --repos-json repos.json --lock-json repos.json.lock --output flake.nix --check

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nur-flake")]
#[command(version = "0.1.0")]
#[command(about = "Generate flake.nix for NUR combined workspace from repos.json")]
struct Args {
    /// Path to repos.json
    #[arg(long, default_value = "repos.json")]
    repos_json: PathBuf,

    /// Path to repos.json.lock
    #[arg(long, default_value = "repos.json.lock")]
    lock_json: PathBuf,

    /// Output path for generated flake.nix
    #[arg(long, default_value = "flake.nix")]
    output: PathBuf,

    /// Check existing flake.nix without overwriting (exit 0 if up-to-date)
    #[arg(long)]
    check: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let generator = cargo_vendormod::nur_flake::NurFlakeGenerator::new(
        args.repos_json.clone(),
        args.lock_json.clone(),
        args.output.clone(),
    );

    if args.check {
        let is_up_to_date = generator
            .check()
            .context("Failed to check flake.nix status")?;
        if is_up_to_date {
            println!(
                "✓ {} is up to date",
                args.output.display()
            );
            std::process::exit(0);
        } else {
            eprintln!(
                "✗ {} is out of date — regenerate needed",
                args.output.display()
            );
            eprintln!(
                "  Run: cargo run --bin nur-flake -- --repos-json {} --lock-json {} --output {}",
                args.repos_json.display(),
                args.lock_json.display(),
                args.output.display()
            );
            std::process::exit(1);
        }
    }

    let repo_count = generator
        .generate()
        .context("Failed to generate flake.nix")?;

    println!(
        "✓ Generated {} with {} repos",
        args.output.display(),
        repo_count
    );
    if args.verbose {
        eprintln!("  Run: nix flake check --impure  # to validate");
    }

    Ok(())
}
