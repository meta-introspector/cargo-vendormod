use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct WarmCmd {
    #[command(subcommand)]
    pub cmd: WarmSubCmd,
}

#[derive(Subcommand, Debug)]
pub enum WarmSubCmd {
    /// Add a directory to the warm list
    Add(AddWarmDirArgs),
    /// List the directories in the warm list
    List,
    /// Remove a directory from the warm list
    Remove(RemoveWarmDirArgs),
    /// Show the status of the warming process
    Status,
}

#[derive(Parser, Debug)]
pub struct AddWarmDirArgs {
    /// Path to the directory
    pub path: PathBuf,
    /// Prefix for the directory in the cache
    pub prefix: String,
    /// File extensions to include (comma-separated)
    #[arg(long)]
    pub exts: Option<String>,
    /// Maximum file size to include
    #[arg(long, default_value = "1048576")]
    pub max_size: u64,
}

#[derive(Parser, Debug)]
pub struct RemoveWarmDirArgs {
    /// Path of the directory to remove
    pub path: PathBuf,
}
