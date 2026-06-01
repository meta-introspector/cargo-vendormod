use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
struct ProjectMetrics {
    total_lines: usize,
    source_lines: usize,
    test_lines: usize,
    total_files: usize,
    source_files: usize,
    test_files: usize,
    // Optional: more detailed metrics
    function_count: usize,
    test_function_count: usize,
}

impl ProjectMetrics {
    fn new() -> Self {
        ProjectMetrics {
            total_lines: 0,
            source_lines: 0,
            test_lines: 0,
            total_files: 0,
            source_files: 0,
            test_files: 0,
            function_count: 0,
            test_function_count: 0,
        }
    }

    fn analyze_project_structure(&mut self) -> io::Result<()> {
        let project_root = ".";
        let test_dirs = vec!["tests", "src/tests"];
        let source_dirs = vec!["src", "src/bin"];

        // Analyze source files
        for dir in source_dirs {
            if Path::new(dir).exists() {
                self.analyze_directory(dir, false)?;
            }
        }

        // Analyze test files
        for dir in test_dirs {
            if Path::new(dir).exists() {
                self.analyze_directory(dir, true)?;
            }
        }

        Ok(())
    }

    fn analyze_directory(&mut self, path: &str, is_test: bool) -> io::Result<()> {
        if !Path::new(path).exists() {
            return Ok(());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.analyze_directory(path.to_str().unwrap_or(""), is_test)?;
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "rs" {
                    // Count the file
                    self.total_files += 1;
                    if is_test {
                        self.test_files += 1;
                    } else {
                        self.source_files += 1;
                    }

                    // Analyze the file content
                    let content = fs::read_to_string(&path)?;
                    let lines: Vec<&str> = content.lines().collect();
                    let mut line_count = 0;
                    let mut function_count = 0;

                    for line in lines {
                        let trimmed = line.trim();
                        // Skip empty lines
                        if trimmed.is_empty() {
                            continue;
                        }
                        // Skip comment-only lines (// or /* ... */ but we'll do simple)
                        if trimmed.starts_with("//") {
                            continue;
                        }
                        // We'll count this as a code line
                        line_count += 1;

                        // Count function definitions (approximate)
                        if trimmed.starts_with("fn ") || 
                           (trimmed.starts_with("pub ") && trimmed.contains("fn ")) ||
                           (trimmed.starts_with("pub(crate) ") && trimmed.contains("fn ")) {
                            function_count += 1;
                        }
                    }

                    // Add to totals
                    if is_test {
                        self.test_lines += line_count;
                        self.test_function_count += function_count;
                    } else {
                        self.source_lines += line_count;
                        self.function_count += function_count;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_test_to_source_ratio(&self) -> f64 {
        if self.source_lines > 0 {
            (self.test_lines as f64 / self.source_lines as f64) * 100.0
        } else {
            0.0
        }
    }

    fn get_test_to_source_file_ratio(&self) -> f64 {
        if self.source_files > 0 {
            (self.test_files as f64 / self.source_files as f64) * 100.0
        } else {
            0.0
        }
    }

    fn render_analysis(&self) -> io::Result<()> {
        println!("📊 Project Code Metrics Analysis");
        println!("===============================");
        println!("Analyzing actual project source and test files...");
        println!();

        println!("📁 File Counts:");
        println!("   Total .rs files: {}", self.total_files);
        println!("   Source files: {}", self.source_files);
        println!("   Test files: {}", self.test_files);
        println!();

        println!("📈 Line Counts:");
        println!("   Total lines: {}", self.total_lines);
        println!("   Source lines: {}", self.source_lines);
        println!("   Test lines: {}", self.test_lines);
        println!();

        println!("📊 Ratios (Lines):");
        println!("   Test-to-Source Ratio: {:.1}%", self.get_test_to_source_ratio());
        println!("   (Test lines ÷ Source lines × 100)");
        println!();

        println!("📊 Ratios (Files):");
        println!("   Test-to-Source File Ratio: {:.1}%", self.get_test_to_source_file_ratio());
        println!("   (Test files ÷ Source files × 100)");
        println!();

        println!("🔢 Function Counts (approximate):");
        println!("   Total functions: {}", self.function_count);
        println!("   Source functions: {}", self.function_count - self.test_function_count);
        println!("   Test functions: {}", self.test_function_count);
        println!();

        println!();
        println!("ℹ️  Notes:");
        println!("   - Line counts exclude empty lines and lines that are only comments (starting with //).");
        println!("   - Function counts are approximate (based on lines starting with 'fn ' or 'pub fn ', etc.).");
        println!("   - This tool does NOT measure actual test coverage or test execution success.");
        println!("   - For real coverage measurement, use tools like cargo-tarpaulin or grcov.");
        println!();

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut metrics = ProjectMetrics::new();

    match metrics.analyze_project_structure() {
        Ok(_) => {
            metrics.render_analysis()?;
        }
        Err(e) => {
            eprintln!("Error analyzing project: {}", e);
            return Err(e);
        }
    }

    Ok(())
}