use anyhow::Result;
use crate::config::{Config, WarmDir};
use crate::warm_args::{AddWarmDirArgs, RemoveWarmDirArgs, WarmSubCmd};

pub fn handle_warm_command(warm_cmd: &WarmSubCmd, config: &mut Config) -> Result<()> {
    match warm_cmd {
        WarmSubCmd::Add(args) => add_warm_dir(args, config),
        WarmSubCmd::List => list_warm_dirs(config),
        WarmSubCmd::Remove(args) => remove_warm_dir(args, config),
        WarmSubCmd::Status => warm_status(config),
    }
}

fn add_warm_dir(args: &AddWarmDirArgs, config: &mut Config) -> Result<()> {
    let new_dir = WarmDir {
        path: args.path.clone(),
        prefix: args.prefix.clone(),
        exts: args.exts.as_ref().map(|s| s.split(',').map(|s| s.to_string()).collect()).unwrap_or_default(),
        max_size: args.max_size,
        last_warmed: None,
    };
    config.warm_dirs.push(new_dir);
    config.save()?;
    println!("Added directory to warm list: {}", args.path.display());
    Ok(())
}

fn list_warm_dirs(config: &Config) -> Result<()> {
    println!("Directories to warm:");
    for dir in &config.warm_dirs {
        println!("- Path: {}", dir.path.display());
        println!("  Prefix: {}", dir.prefix);
        println!("  Extensions: {}", dir.exts.join(", "));
        println!("  Max size: {}", dir.max_size);
    }
    Ok(())
}

fn remove_warm_dir(args: &RemoveWarmDirArgs, config: &mut Config) -> Result<()> {
    config.warm_dirs.retain(|d| d.path != args.path);
    config.save()?;
    println!("Removed directory from warm list: {}", args.path.display());
    Ok(())
}

fn warm_status(config: &Config) -> Result<()> {
    println!("Warming status:");
    for dir in &config.warm_dirs {
        println!("- Path: {}", dir.path.display());
        println!("  Prefix: {}", dir.prefix);
        println!("  Extensions: {}", dir.exts.join(", "));
        println!("  Max size: {}", dir.max_size);
        println!("  Last warmed: {}", dir.last_warmed.as_deref().unwrap_or("Never"));
    }
    Ok(())
}
