use krates::{Builder, Cmd, Krates};

fn main() -> Result<(), krates::Error> {
    // Example: Build a crate graph from a Cargo.toml file
    let mut cmd = Cmd::new();
    cmd.manifest_path("../krates/Cargo.toml");
    
    let builder = Builder::new();
    
    // Build the crate graph
    let krates: Krates = builder.build(cmd, |pkg: krates::cm::Package| {
        println!("Filtering out crate: {}", pkg.id);
    })?;
    
    println!("Built crate graph with {} crates", krates.len());
    
    // Print some basic info about the graph
    for krate in krates.krates() {
        println!("- {} v{}", krate.name, krate.version);
    }
    
    Ok(())
}