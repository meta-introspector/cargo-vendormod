//! # Language Detection Module
//!
//! Detects the programming language of a project by inspecting its manifest files.
//! Used by the flake generator to select the appropriate build recipe.
//!
//! ## Detection Order
//!
//! Detects manifests in priority order. A project may have multiple manifest files
//! (e.g. `pyproject.toml` in a Rust project wrapping Python bindings), so we check
//! the most specific first:
//!
//! 1. `Cargo.toml` with `[package]` → Rust
//! 2. `go.mod` → Go
//! 3. `package.json` → JavaScript / Node
//! 4. `pyproject.toml` → Python
//! 5. `pom.xml` or `build.gradle` → Java
//! 6. `setup.py` or `setup.cfg` → Python (legacy)
//! 7. `CMakeLists.txt` → C/C++ (CMake)
//! 8. `Makefile` or `makefile` → C/C++ (Make)
//! 9. `*.go` files present → Go (heuristic)
//! 10. `*.c`, `*.cpp`, `*.h` files present → C/C++ (heuristic)

use std::path::{Path, PathBuf};

/// Programming languages supported by the flake generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Rust (Cargo.toml with [package])
    Rust,
    /// Go (go.mod)
    Go,
    /// Python (pyproject.toml, setup.py, setup.cfg)
    Python,
    /// JavaScript / Node.js (package.json)
    JavaScript,
    /// Java / JVM (pom.xml, build.gradle)
    Java,
    /// C / C++ (CMakeLists.txt, Makefile, or .c/.cpp/.h files)
    C,
    /// Unknown — fallback to generic
    Unknown,
}

impl Language {
    /// Human-readable label for the language.
    pub fn label(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::Java => "Java",
            Language::C => "C/C++",
            Language::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Detect the language of a project at the given directory.
///
/// Checks for manifest files in priority order. Returns `Unknown` if no
/// recognized manifest is found.
pub fn detect_language(project_dir: &Path) -> Language {
    // 1. Rust: Cargo.toml with [package] section
    let cargo_path = project_dir.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if content.contains("[package]") {
                return Language::Rust;
            }
        }
    }

    // 2. Go: go.mod
    if project_dir.join("go.mod").exists() {
        return Language::Go;
    }

    // 3. JavaScript: package.json
    if project_dir.join("package.json").exists() {
        return Language::JavaScript;
    }

    // 4. Python: pyproject.toml
    if project_dir.join("pyproject.toml").exists() {
        return Language::Python;
    }

    // 5. Java: pom.xml or build.gradle
    if project_dir.join("pom.xml").exists() || project_dir.join("build.gradle").exists() {
        return Language::Java;
    }

    // 6. Python (legacy): setup.py or setup.cfg
    if project_dir.join("setup.py").exists() || project_dir.join("setup.cfg").exists() {
        return Language::Python;
    }

    // 7. C/C++: CMakeLists.txt
    if project_dir.join("CMakeLists.txt").exists() {
        return Language::C;
    }

    // 8. C/C++: Makefile
    if project_dir.join("Makefile").exists() || project_dir.join("makefile").exists() {
        return Language::C;
    }

    // 9. Heuristic: check for source file extensions
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        let mut has_go = false;
        let mut has_c = false;
        let mut has_cpp = false;

        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".go") {
                        has_go = true;
                    }
                    if name.ends_with(".c") || name.ends_with(".h") {
                        has_c = true;
                    }
                    if name.ends_with(".cpp") || name.ends_with(".cc") || name.ends_with(".cxx") {
                        has_cpp = true;
                    }
                }
            }
        }

        if has_go {
            return Language::Go;
        }
        if has_c || has_cpp {
            return Language::C;
        }
    }

    Language::Unknown
}

/// Recursively discover projects under a root directory.
///
/// Walks the directory tree up to `max_depth` and returns all directories
/// that contain a recognizable project manifest (Cargo.toml, go.mod, etc.).
///
/// Returns a vector of `(PathBuf, Language)` pairs.
pub fn discover_projects(root: &Path, max_depth: usize) -> Vec<(PathBuf, Language)> {
    let mut results = Vec::new();
    _discover_recursive(root, root, max_depth, &mut results);
    results
}

fn _discover_recursive(base: &Path, current: &Path, depth: usize, results: &mut Vec<(PathBuf, Language)>) {
    if depth == 0 {
        return;
    }

    // Check if current directory is a recognized project
    let lang = detect_language(current);
    if lang != Language::Unknown {
        results.push((current.to_path_buf(), lang));
        return; // Don't recurse into recognized projects (they might have sub-projects)
    }

    // Recurse into subdirectories
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                // Skip hidden directories and common non-project dirs
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') || name == "target" || name == "node_modules"
                        || name == "__pycache__" || name == ".lake"
                    {
                        continue;
                    }
                }
                _discover_recursive(base, &entry.path(), depth - 1, results);
            }
        }
    }
}

/// Get a human-readable description of what manifest was found (for verbose logging).
pub fn describe_manifest(project_dir: &Path) -> String {
    let checks = [
        ("Cargo.toml", project_dir.join("Cargo.toml")),
        ("go.mod", project_dir.join("go.mod")),
        ("package.json", project_dir.join("package.json")),
        ("pyproject.toml", project_dir.join("pyproject.toml")),
        ("pom.xml", project_dir.join("pom.xml")),
        ("CMakeLists.txt", project_dir.join("CMakeLists.txt")),
        ("Makefile", project_dir.join("Makefile")),
        ("setup.py", project_dir.join("setup.py")),
    ];

    for (name, path) in &checks {
        if path.exists() {
            return name.to_string();
        }
    }
    "no recognized manifest".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_rust() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Rust);
    }

    #[test]
    fn test_detect_go() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "module test\ngo 1.21\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Go);
    }

    #[test]
    fn test_detect_python() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Python);
    }

    #[test]
    fn test_detect_javascript() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{\"name\": \"test\"}\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::JavaScript);
    }

    #[test]
    fn test_detect_java() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pom.xml"), "<project></project>").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Java);
    }

    #[test]
    fn test_detect_c_cmake() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("CMakeLists.txt"), "cmake_minimum_required(VERSION 3.0)\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::C);
    }

    #[test]
    fn test_detect_c_heuristic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.c"), "int main() { return 0; }\n").unwrap();
        assert_eq!(detect_language(dir.path()), Language::C);
    }

    #[test]
    fn test_detect_unknown() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();
        assert_eq!(detect_language(dir.path()), Language::Unknown);
    }

    #[test]
    fn test_describe_manifest() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "").unwrap();
        assert_eq!(describe_manifest(dir.path()), "go.mod");
    }
}
