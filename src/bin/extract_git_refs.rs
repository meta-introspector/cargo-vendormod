use std::fs;
use std::path::Path;
use regex::Regex;
use std::collections::HashSet;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = args.get(1).map(|s| s.as_str()).unwrap_or("newroot_bg/newroot_bg");
    let output = args.get(2).map(|s| s.as_str()).unwrap_or("all_git_refs.txt");
    
    println!("Scanning for git references in: {}", root);
    
    let mut all_refs = HashSet::new();
    let git_url_regex = Regex::new(r#"https?://[a-zA-Z0-9._/-]+/([a-zA-Z0-9_-]+)/([a-zA-Z0-9._-]+)(?:\.git)?"#).unwrap();
    
    scan_directory(Path::new(root), &git_url_regex, &mut all_refs);
    
    let mut refs: Vec<_> = all_refs.into_iter().collect();
    refs.sort();
    
    fs::write(output, refs.join("\n")).unwrap();
    println!("Found {} unique git references, saved to {}", refs.len(), output);
}

fn scan_directory(dir: &Path, regex: &Regex, refs: &mut HashSet<String>) {
    if !dir.is_dir() {
        return;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            if name == ".git" || name.starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                if path.join("Cargo.toml").exists() {
                    scan_cargo_toml(&path, regex, refs);
                }
                scan_directory(&path, regex, refs);
            }
        }
    }
}

fn scan_cargo_toml(dir: &Path, regex: &Regex, refs: &mut HashSet<String>) {
    let toml_path = dir.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&toml_path) {
        for cap in regex.find_iter(&content) {
            let url = cap.as_str();
            if url.contains("github.com") || url.contains("gitlab.com") || url.contains("bitbucket.org") {
                refs.insert(url.to_string());
            }
        }
    }
    
    if let Ok(lock_content) = fs::read_to_string(dir.join("Cargo.lock")) {
        for cap in regex.find_iter(&lock_content) {
            let url = cap.as_str();
            if url.contains("github.com") || url.contains("gitlab.com") || url.contains("bitbucket.org") {
                refs.insert(url.to_string());
            }
        }
    }
}