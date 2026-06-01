use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
struct ProjectFileAnalysis {
    total_files: usize,
    test_files: usize,
    source_files: usize,
    file_categories: HashMap<String, usize>,
    test_categories: HashMap<String, usize>,
}

impl ProjectFileAnalysis {
    fn new() -> Self {
        ProjectFileAnalysis {
            total_files: 0,
            test_files: 0,
            source_files: 0,
            file_categories: HashMap::new(),
            test_categories: HashMap::new(),
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

        // Calculate source files (total - test)
        self.source_files = self.total_files.saturating_sub(self.test_files);

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
                    if is_test {
                        self.test_files += 1;
                        *self.test_categories.entry(ext_str.clone()).or_insert(0) += 1;
                    } else {
                        self.total_files += 1;
                        *self.file_categories.entry(ext_str.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_test_to_source_ratio(&self) -> f64 {
        if self.source_files > 0 {
            (self.test_files as f64 / self.source_files as f64) * 100.0
        } else {
            0.0
        }
    }

    fn get_file_distribution(&self) -> String {
        let mut sorted: Vec<(&String, &usize)> = self.file_categories.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        
        sorted.iter()
            .take(5) // Top 5
            .map(|(ext, count)| format!("{}: {}", ext, count))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn get_test_file_distribution(&self) -> String {
        let mut sorted: Vec<(&String, &usize)> = self.test_categories.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        
        sorted.iter()
            .take(5) // Top 5
            .map(|(ext, count)| format!("{}: {}", ext, count))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn render_analysis(&self) -> io::Result<()> {
        println!("📊 Project File Structure Analysis");
        println!("===============================");
        println!("Analyzing actual project file structure...");
        println!();

        println!("📁 File Counts:");
        println!("   Total .rs files: {}", self.total_files);
        println!("   Source files: {}", self.source_files);
        println!("   Test files: {}", self.test_files);
        println!();

        println!("📈 Test-to-Source Ratio: {:.1}%", self.get_test_to_source_ratio());
        println!("   (Test files ÷ Source files × 100)");
        println!("   ℹ️  This measures test file quantity relative to source files.");
        println!("   ℹ️  It does NOT measure actual test coverage or test execution success.");
        println!();

        if !self.file_categories.is_empty() {
            println!("📂 Top Source File Types: {}", self.get_file_distribution());
        }

        if !self.test_categories.is_empty() {
            println!("🧪 Top Test File Types: {}", self.get_test_file_distribution());
        }

        println!();
        println!("📝 For actual test coverage measurement, consider:");
        println!("   - cargo tarpaulin --all-features");
        println!("   - grcov . -s . -t html --branch --ignore-not-existing");
        println!("   - cargo tarpaulin --out Xml");
        println!();

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut analysis = ProjectFileAnalysis::new();

    match analysis.analyze_project_structure() {
        Ok(_) => {
            analysis.render_analysis()?;
        }
        Err(e) => {
            eprintln!("Error analyzing project: {}", e);
            return Err(e);
        }
    }

    Ok(())
}