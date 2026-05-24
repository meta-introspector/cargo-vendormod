# Scanner System Documentation

## Overview

The scanner system is a comprehensive toolset for analyzing codebases with a focus on git repository discovery and CBOR file analysis using advanced statistical models.

## Architecture

### Core Components

1. **CBOR Scanner** (`scanners/cbor_scanner/`)
   - Byte-level Hidden Markov Model (HMM) analysis
   - First-order Markov model baseline
   - Anomaly detection through statistical analysis
   - Integer constant extraction from CBOR files

2. **Git Scanner** (`scanners/git_scanner/`)
   - Git repository and submodule discovery
   - Repository metadata extraction
   - Language detection and classification
   - Integration with language-specific scanners

3. **Workflow Orchestrator** (`scanners/scan_all.sh`)
   - Unified bash script for complete scanning workflow
   - Builds scanners, discovers repos, extracts data, trains models, detects anomalies

4. **Configuration System** (`scanners/scanner_config.json`)
   - JSON-based scanner configuration
   - Extensible scanner definitions with commands and output directories

## Detailed Analysis

### CBOR Scanner (`scanners/cbor_scanner/src/main.rs`)

#### Core Functionality
- **HMM Implementation**: 
  - State representation with transition and emission probabilities
  - Baum-Welch algorithm placeholder (currently using simple training)
  - Viterbi algorithm for sequence scoring and state path detection
  - 4-state clustering: control bytes (0-31), printable ASCII (32-126), extended bytes (127-255), CBOR major types (0-255)

- **Training Method**:
  - Simple byte frequency analysis and co-occurrence counting
  - State distribution based on byte value ranges
  - Transition probability calculation from byte pair frequencies

- **Analysis Capabilities**:
  - CBOR deserialization for integer extraction
  - Raw byte scanning for embedded integer patterns (1, 2, 4, 8 bytes)
  - Anomaly detection through rare state transitions (< 0.001 probability)
  - JSON output with detailed analysis per file

#### CLI Commands
- `train`: Train HMM on CBOR files with configurable states
- `scan`: Score files using trained HMM and detect anomalies
- `extract`: Extract integer constants from CBOR files

#### Dependencies
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
ciborium = "0.2"
walkdir = "2"
```

### Git Scanner (`scanners/git_scanner/src/main.rs`)

#### Core Functionality
- **Repository Discovery**:
  - Recursive directory walking to find `.git` directories
  - Submodule detection and expansion
  - Git metadata extraction (URL, branch, commit, submodule status)

- **Language Detection**:
  - File extension analysis for 25+ programming languages
  - Build file detection (Cargo.toml, package.json, go.mod)
  - CBOR file identification
  - Language set per repository

- **Scanner Integration**:
  - Configurable scanner execution with command templates
  - Output directory management
  - Cross-platform command execution

#### CLI Commands
- `discover`: Find git repositories with optional recursive submodule expansion
- `scan`: Run configured scanners on discovered repositories
- `list-languages`: Enumerate all programming languages found

#### Dependencies
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
walkdir = "2"
rayon = "1.8"
log = "0.4"
env_logger = "0.10"
```

### Workflow Integration (`scanners/scan_all.sh`)

#### Complete Workflow Steps
1. **Build Phase**: Compile all scanners with `cargo build --release --all`
2. **Discovery Phase**: Find git repositories using git_scanner
3. **Extraction Phase**: Extract constants from CBOR files
4. **Training Phase**: Train HMM models on CBOR data
5. **Analysis Phase**: Scan for anomalies using trained models

#### Key Features
- Color-coded logging (INFO, OK, WARN, ERROR)
- Configurable root directories and output paths
- Error handling with `set -euo pipefail`
- Flexible argument parsing for individual commands or full workflow

### Configuration System (`scanners/scanner_config.json`)

#### Scanner Definitions
```json
{
  "scanners": [
    {
      "name": "cbor_scanner",
      "extensions": ["cbor"],
      "command": "cargo run --bin cbor_scanner -- extract -i {dir} -o {output}/cbor",
      "output_dir": "cbor"
    }
  ]
}
```

#### Configuration Structure
- `output_dir`: Base output directory for all scans
- `scanners`: Array of scanner configurations
  - `name`: Scanner identifier
  - `extensions`: Target file extensions
  - `command`: Execution template with {dir} and {output} placeholders
  - `output_dir`: Subdirectory for scanner outputs

## Usage Examples

### Basic CBOR Analysis
```bash
# Train HMM on CBOR files
cargo run --bin cbor_scanner -- train -i ./data -o model.json -s 8

# Scan files for anomalies
cargo run --bin cbor_scanner -- scan -i ./data -m model.json -o analysis/

# Extract constants
cargo run --bin cbor_scanner -- extract -i ./data -o constants.json
```

### Git Repository Discovery
```bash
# Discover repositories
cargo run --bin git_scanner -- discover -i ./repos -o repos.json --recursive

# List languages found
cargo run --bin git_scanner -- list-languages -i repos.json

# Scan repositories with configured scanners
cargo run --bin git_scanner -- scan -i repos.json -o scan_output
```

### Complete Workflow
```bash
# Run full scanning workflow
./scan_all.sh full ./workspace ./output

# Individual workflow steps
./scan_all.sh build
./scan_all.sh discover ./workspace
./scan_all.sh extract ./repos.json ./analysis
```

## Technical Implementation Details

### HMM Algorithm Details
- **State Definition**: Each state represents a byte value range with emission probabilities
- **Training**: Simple frequency-based clustering (Baum-Welch planned for future)
- **Scoring**: Viterbi algorithm with log probabilities for numerical stability
- **Anomaly Detection**: Flags transitions with probability < 0.001

### Git Processing
- **Submodule Detection**: Checks for `.git` files containing "gitdir:" references
- **Metadata Extraction**: Reads git config files and executes git commands
- **Language Detection**: Extensible mapping of file extensions to languages

### Error Handling
- Comprehensive error propagation using `anyhow::Result`
- Graceful handling of missing files and invalid configurations
- Logging integration with structured output

## Performance Considerations

- **Memory Usage**: HMM models scale linearly with number of states and training data
- **Processing Speed**: Byte-level analysis is O(n) for file scanning
- **Parallel Processing**: Git scanner uses rayon for potential parallelization
- **Disk I/O**: Efficient file walking and JSON serialization

## Extensibility

### Adding New Scanners
1. Create new scanner binary with CLI interface
2. Add configuration to `scanner_config.json`
3. Implement scanner integration in git_scanner
4. Update workflow script if needed

### Extending Language Detection
- Add new file extension mappings in `detect_languages()`
- Include additional build file checks
- Extend language list in configuration

### Enhancing HMM Models
- Implement Baum-Welch algorithm for better training
- Add more sophisticated state definitions
- Include context-aware transition modeling

## Future Enhancements

### Planned Improvements
1. **Advanced HMM Training**: Implement Baum-Welch algorithm
2. **Multi-format Support**: Extend beyond CBOR to other binary formats
3. **Machine Learning Integration**: Add ML-based anomaly detection
4. **Performance Optimization**: Parallel processing for large datasets
5. **Web Interface**: Add REST API for remote scanning

### Integration Opportunities
- CI/CD pipeline integration for automated scanning
- IDE plugin for real-time code analysis
- Cloud-based scanning service
- Integration with static analysis tools

## Security Considerations

- **Input Validation**: Comprehensive path validation and sanitization
- **Command Execution**: Safe command template substitution
- **File Permissions**: Proper handling of file access permissions
- **Data Privacy**: Consideration for sensitive code analysis

## Troubleshooting

### Common Issues
1. **Missing Dependencies**: Ensure all Rust dependencies are installed
2. **Permission Errors**: Check file system permissions for target directories
3. **Git Repository Issues**: Verify git commands work in target directories
4. **Configuration Errors**: Validate JSON syntax and file paths

### Debug Mode
- Enable verbose logging with `RUST_LOG=debug` environment variable
- Use individual commands instead of full workflow for step-by-step debugging
- Check scanner output directories for detailed error information

## Conclusion

The scanner system provides a robust, extensible foundation for codebase analysis with a focus on statistical modeling of binary data and comprehensive git repository management. The modular architecture allows for easy extension and integration with other analysis tools while maintaining high performance and reliability.