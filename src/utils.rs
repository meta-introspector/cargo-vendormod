
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
    }
}
