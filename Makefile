# Cargo Vendormod Makefile for Solana Processing

.PHONY: help solana-process solana-report clean projects-graphs projects-summary global-graph build-order upgrade-plan bare-repos

# Configuration
SOLANA_CRATES := solana_crates.txt
ALL_CRATES := all_cargo_tomls.txt
CARGO_VENDORMOD := target/release/cargo-vendormod
SOLANA_OUTPUT := solana_processing
FINAL_OUTPUT := final_processing
RESULTS_DIR := test_results
DATE := $(shell date +%Y%m%d_%H%M%S)
PROJECTS_DIR := /home/mdupont/projects
PROJECTS_OUTPUT := projects_graphs
GLOBAL_GRAPH := global_graph

# Analyze all Rust projects in ~/projects/
projects-graphs:
	@echo "🔍 Analyzing all Rust projects in $(PROJECTS_DIR)..."
	@mkdir -p $(PROJECTS_OUTPUT)
	@for repo in $(PROJECTS_DIR)/*/; do \
		name=$$(basename $$repo); \
		if [ -f "$$repo/Cargo.toml" ]; then \
			echo "=== $$name ==="; \
			mkdir -p $(PROJECTS_OUTPUT)/$$name; \
			nix run .#analyze-repo -- $$repo $(PROJECTS_OUTPUT)/$$name 2>&1; \
		else \
			echo "SKIP (no Cargo.toml): $$name" > $(PROJECTS_OUTPUT)/$$name/status.txt 2>/dev/null || true; \
		fi; \
	done
	@echo "✅ All projects analyzed!"
	@echo "📊 Results in $(PROJECTS_OUTPUT)/"

# Summarize all project graph analyses
projects-summary:
	@echo "📊 === Project Graph Analysis Summary ==="
	@echo ""
	@for dir in $(PROJECTS_OUTPUT)/*/; do \
		name=$$(basename $$dir); \
		if [ -f "$$dir/summary.txt" ]; then \
			summary=$$(cat $$dir/summary.txt 2>/dev/null); \
			printf "  %-30s %s\n" "$$name" "$$summary"; \
		elif [ -f "$$dir/status.txt" ]; then \
			status=$$(cat $$dir/status.txt 2>/dev/null); \
			printf "  %-30s %s\n" "$$name" "$$status"; \
		else \
			printf "  %-30s %s\n" "$$name" "no results"; \
		fi; \
	done
	@echo ""
	@echo "Total projects: $$(ls -d $(PROJECTS_OUTPUT)/*/ 2>/dev/null | wc -l)"
	@echo "Analyzed: $$(find $(PROJECTS_OUTPUT) -name summary.txt 2>/dev/null | wc -l)"

# Build global graph from all project graphs
global-graph: $(PROJECTS_OUTPUT)
	@mkdir -p $(GLOBAL_GRAPH)
	@./target/debug/graph merge -i $(PROJECTS_OUTPUT) -o $(GLOBAL_GRAPH)

# Show build order (top 20)
build-order: global-graph
	@python3 -c "import json; d=json.load(open('$(GLOBAL_GRAPH)/build_order.json')); print(f'Total: {d[\"total_crates\"]} crates in build order'); [print(f'  {i+1}. {o[\"crate\"]}') for i, o in enumerate(d['order'][:20])]"

# Show upgrade plan summary (version mismatches)
upgrade-plan: global-graph
	@python3 -c "import json; d=json.load(open('$(GLOBAL_GRAPH)/upgrade_plan.json')); print(f'Upgrade candidates: {d[\"total_candidates\"]}'); [print(f'  {p[\"crate\"]}: {p[\"oldest\"]} -> {p[\"newest\"]}') for p in d['plans'][:10]]"

# Show bare repo mirrors
bare-repos: global-graph
	@echo "=== Bare Git Repos (local mirrors) ==="
	@ls -1 $(GLOBAL_GRAPH)/bare_repos/ 2>/dev/null | sed 's/^/  /' || echo "  (no bare repos)"
	@echo "Cargo mirror config: $(GLOBAL_GRAPH)/cargo_mirror_config.toml"

# Build all projects via crate2nix (each crate = separate Nix store derivation)
crate2nix-build-%:
	@echo "🔨 Building $* via crate2nix..."
	nix build --impure ./crate2nix_output#$* -o /tmp/nix_builds/$*
	@echo "✅ $* built: $$(readlink /tmp/nix_builds/$*)"

# Build all successful crate2nix projects
crate2nix-build-all:
	@mkdir -p /tmp/nix_builds
	@for proj in $$(ls -d crate2nix_output/*/ 2>/dev/null | sed 's|crate2nix_output/||;s|/||' | grep -v flake); do \
		printf "🔨 Building %-30s ... " "$$proj"; \
		nix build --impure ./crate2nix_output#$$proj -o /tmp/nix_builds/$$proj 2>&1 | tail -1; \
		[ -L /tmp/nix_builds/$$proj ] && echo "✅ $$(readlink /tmp/nix_builds/$$proj)" || echo "❌ FAILED"; \
	done
	@echo "📊 Build results in /tmp/nix_builds/"

# Default target - Comprehensive Solana vendoring workflow
solana:
	@echo "🚀 Starting comprehensive Solana vendoring..."
	@echo "============================================"

# Build the tool first
	@echo "🔨 Building cargo-vendormod..."
	cargo build --release

# Process Solana SDK Layer 1 (external dependencies)
	@echo "📦 Processing Solana SDK external dependencies..."
	nix develop --command ./target/release/cargo-vendormod process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
	    --output-dir solana_processing/solana_sdk --layered-processing --layer 1

# Process Solana SDK Layer 2 (workspace members)
	@echo "🏗️ Processing Solana SDK workspace members..."
	nix develop --command ./target/release/cargo-vendormod process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
	    --output-dir solana_processing/solana_sdk --layered-processing --layer 2

# Process Solana Main Layer 1 (external dependencies)
	@echo "📦 Processing Solana Main external dependencies..."
	nix develop --command ./target/release/cargo-vendormod process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main \
	    --output-dir solana_processing/solana_main --layered-processing --layer 1

# Process Solana Main Layer 2 (workspace members)
	@echo "🏗️ Processing Solana Main workspace members..."
	nix develop --command ./target/release/cargo-vendormod process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main \
	    --output-dir solana_processing/solana_main --layered-processing --layer 2

	@echo "✅ Solana vendoring complete!"
	@echo "📊 Final status:"
	@echo "  Total flakes: $$(find solana_processing/ -name \"flake.nix\" | wc -l)"
	@echo "  SDK workspace: $$(ls solana_processing/solana_sdk/layer2/ | wc -l)/186"
	@echo "  Main workspace: $$(ls solana_processing/solana_main/layer2/ | wc -l)/189"

help:
	@echo "🚀 Cargo Vendormod Makefile"
	@echo "========================================"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  solana              - COMPREHENSIVE Solana vendoring (DEFAULT - does everything!)"
	@echo "  solana-process      - Process all Solana crates (with Nix env)"
	@echo "  solana-sdk-layer1   - Process Solana SDK external dependencies (Nix)"
	@echo "  solana-sdk-layer2   - Process Solana SDK workspace members (Nix)"
	@echo "  solana-main-layer1  - Process Solana Main external dependencies (Nix)"
	@echo "  solana-main-layer2  - Process Solana Main workspace members (Nix)"
	@echo "  solana-report       - Generate Solana processing report"
	@echo "  final-process       - Process remaining crates"
	@echo "  full-process        - Process everything (Solana + remaining)"
	@echo "  test-all            - Run ALL tools and upload results to pastebinit"
	@echo "  test-atlas          - Run mathematical atlas tools"
	@echo "  test-coverage      - Run performance coverage tools"
	@echo "  test-suite          - Run test suite"
	@echo "  upload-results      - Upload results to pastebinit"
	@echo "  projects-graphs      - Analyze all Rust projects in ~/projects/"
	@echo "  projects-summary     - Summarize project graph analyses"
	@echo "  global-graph         - Merge all project graphs into one global dependency graph"
	@echo "  build-order          - Show top 20 crates in global build order"
	@echo "  upgrade-plan         - Show upgrade candidates (version mismatches)"
	@echo "  bare-repos           - Show bare repo mirror status"
	@echo "  clean               - Clean build artifacts"
	@echo "  build               - Build cargo-vendormod"
	@echo "  help                - Show this help"
	@echo ""

# Build the tool
build:
	@echo "🔨 Building cargo-vendormod..."
	cargo build --release
	@echo "✅ Build complete!"

# Process Solana crates (optimized workspace approach)
solana-process: build
	@echo "🚀 Processing Solana SDK workspace..."
	mkdir -p $(SOLANA_OUTPUT)
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
	    --output-dir $(SOLANA_OUTPUT)/solana_sdk --layered-processing
	@echo "✅ Solana SDK processing complete!"
	@echo "🚀 Processing Solana main workspace..."
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main \
	    --output-dir $(SOLANA_OUTPUT)/solana_main --layered-processing
	@echo "✅ Solana main processing complete!"

# Process specific layers for Solana SDK
solana-sdk-layer1:
	@echo "🔧 Processing Solana SDK Layer 1 (external dependencies) with Nix..."
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
	    --output-dir $(SOLANA_OUTPUT)/solana_sdk --layered-processing --layer 1
	@echo "✅ Solana SDK Layer 1 complete!"

solana-sdk-layer2:
	@echo "🔧 Processing Solana SDK Layer 2 (workspace members) with Nix..."
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk \
	    --output-dir $(SOLANA_OUTPUT)/solana_sdk --layered-processing --layer 2
	@echo "✅ Solana SDK Layer 2 complete!"

# Process specific layers for Solana Main
solana-main-layer1:
	@echo "🔧 Processing Solana Main Layer 1 (external dependencies) with Nix..."
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main \
	    --output-dir $(SOLANA_OUTPUT)/solana_main --layered-processing --layer 1
	@echo "✅ Solana Main Layer 1 complete!"

solana-main-layer2:
	@echo "🔧 Processing Solana Main Layer 2 (workspace members) with Nix..."
	nix develop --command $(CARGO_VENDORMOD) process-crates \
	    /mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main \
	    --output-dir $(SOLANA_OUTPUT)/solana_main --layered-processing --layer 2
	@echo "✅ Solana Main Layer 2 complete!"

# Process remaining crates
final-process: build
	@echo "🔄 Processing remaining crates..."
	mkdir -p $(FINAL_OUTPUT)
	$(CARGO_VENDORMOD) process-all-crates $(ALL_CRATES) --output-dir $(FINAL_OUTPUT) --max-parallel 8
	@echo "✅ Final processing complete!"

# Process everything
full-process: build
	@echo "🌐 Processing everything..."
	make solana-process
	make final-process
	@echo "✅ Full processing complete!"

# Generate Solana report
solana-report:
	@echo "📊 Generating Solana processing report..."
	$(CARGO_VENDORMOD) generate-report $(SOLANA_CRATES) --output-dir $(SOLANA_OUTPUT)
	@echo "✅ Report generated!"

# Clean targets
clean:
	@echo "🧹 Cleaning..."
	 rm -rf $(SOLANA_OUTPUT) $(FINAL_OUTPUT)
	cargo clean
	@echo "✅ Clean complete!"

# Quick status check
status:
	@echo "📊 Current Status:"
	@echo "  Total flakes generated: $$(find $(SOLANA_OUTPUT) -name \"flake.nix\" 2>/dev/null | wc -l)"
	@echo ""
	@echo "Solana SDK:"
	@echo "  Layer 1 (external): $$(ls $(SOLANA_OUTPUT)/solana_sdk/layer1/ 2>/dev/null | wc -l)/93"
	@echo "  Layer 2 (workspace): $$(ls $(SOLANA_OUTPUT)/solana_sdk/layer2/ 2>/dev/null | wc -l)/186"
	@echo ""
	@echo "Solana Main:"
	@echo "  Layer 1 (external): $$(ls $(SOLANA_OUTPUT)/solana_main/layer1/ 2>/dev/null | wc -l)/218"
	@echo "  Layer 2 (workspace): $$(ls $(SOLANA_OUTPUT)/solana_main/layer2/ 2>/dev/null | wc -l)/189"

# New bootstrap target to vendoring cargo-vendormod itself
.PHONY: bootstrap
bootstrap: build
	@echo "🚀 Bootstrapping cargo-vendormod vendoring itself..."
	@mkdir -p newroot/vendor/submodules newroot/vendor/mirrors
	@echo "Setting up submodules in newroot..."
	@cd newroot && (git submodule status | grep -q . || (git submodule add https://github.com/meta-introspector/cargo-edit cargo-edit && \
	  git submodule add https://github.com/meta-introspector/krates krates && \
	  git submodule add https://github.com/loadingalias/cargo-rail workload/workspaces/cargo-rail && \
	  git submodule add https://github.com/meta-introspector/zkperf zkperf))
	@echo "Running vendoring with discovery..."
	cargo run --release --bin cargo-vendormod -- \
	  --manifest-path ./Cargo.toml \
	  --submodules-path newroot/vendor/submodules \
	  --mirrors-path newroot/vendor/mirrors \
	  --root-dir newroot \
	  --source-repo newroot \
	  --include-dev \
	  --include-build \
	  --include-optional \
	  vendoring
	@echo "✅ Bootstrap vendoring complete!"

# Target to extract all crate info from Cargo.lock (including registry crates from crates.io)
.PHONY: crate-info
crate-info: build
	@echo "📦 Extracting all crate info from Cargo.lock..."
	@echo "This will query crates.io API for repository URLs..."
	cargo run --release --bin crate_info -- . 2>/dev/null || \
	cargo run --release --bin cargo-vendormod -- \
	  --manifest-path ./Cargo.toml \
	  --submodules-path newroot/vendor/submodules \
	  --mirrors-path newroot/vendor/mirrors \
	  --root-dir . \
	  --include-dev \
	  --include-build \
	  --include-optional \
	  vendoring 2>&1 | head -50
	@echo "✅ Crate info extraction complete!"

# Run bootstrap and then show all crate info
.PHONY: bootstrap-crate-info
bootstrap-crate-info: bootstrap
	@echo ""
	@echo "=== All Vendored Repositories ==="
	@find newroot/vendor/submodules -name ".git" -type d -exec dirname {} \; 2>/dev/null | head -20

# Show all crates with repository URLs (fetches from crates.io API)
.PHONY: show-crates
show-crates: build
	@echo "📦 Fetching repository URLs for all crates in Cargo.lock..."
	@echo ""
	@echo "This queries crates.io API to get repository URLs for each crate"
	@echo "..."
	@echo "You can also run: cargo run --release --bin crate_info -- ."
	@echo ""
	@cargo run --release --bin crate_info -- . 2>&1 | grep -E "(GitHub|Registry|Path|Summary)" -A 100

# Fetch and display all crate repo URLs from crates.io
.PHONY: fetch-crates-io-repos
fetch-crates-io-repos: build
	@echo "🔍 Fetching repository URLs from crates.io for all crates..."
	@echo ""
	cargo run --release --bin cargo-vendormod -- \
	  --manifest-path ./Cargo.toml \
	  --submodules-path newroot/vendor/submodules \
	  --mirrors-path newroot/vendor/mirrors \
	  --root-dir . \
	  --source-repo . \
	  --fetch-crates-io-repos \
	  vendoring 2>&1 | head -60


new_test:
new_test:
#	/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod && cargo build 2>&1 | tail -3 &&
	cargo run --bin cargo-vendormod -- extract-all --workspace-path ~/projects/agave-solana-validator --newroot ~/projects/solana-vendored-test

# Mathematical Atlas and Performance Coverage Testing
.PHONY: test-all test-atlas test-coverage test-suite upload-results unit-test

# Run unit tests using nix develop
unit-test:
	@echo "🔬 Running cargo-vendormod unit tests..."
	nix develop -f ../../flake.nix -c cargo test --package cargo-vendormod

unit-test-no-default:
	@echo "🔬 Running cargo-vendormod unit tests (no default features)..."
	nix develop -f ../../flake.nix -c cargo test --package cargo-vendormod --no-default-features

unit-test-verbose:
	@echo "🔬 Running cargo-vendormod unit tests (verbose)..."
	nix develop -f ../../flake.nix -c cargo test --package cargo-vendormod -- --nocapture --test-threads=1

unit-check:
	@echo "🔍 Checking cargo-vendormod code..."
	nix develop -f ../../flake.nix -c cargo check --package cargo-vendormod --all-features

unit-clippy:
	@echo "📎 Running clippy on cargo-vendormod..."
	nix develop -f ../../flake.nix -c cargo clippy --package cargo-vendormod --all-features -- -D warnings

# Run cargo test coverage report
cargo-test-coverage:
	@echo "📊 Running test coverage analysis..."
	nix develop -f ../../flake.nix -c cargo test --package cargo-vendormod --all-features 2>&1 | tee target/test_output.log
	@echo "Coverage report generated in target/"

# Run workload processor to analyze git repos and Cargo.toml files
workload: build
	@echo "📊 Running workload processor..."
	nix develop -f ../../flake.nix -c cargo run --bin workload_processor -- .

# Run workload benchmark with detailed timing
benchmark-workload:
	@echo "⏱️  Running workload benchmark..."
	time nix develop -f ../../flake.nix -c cargo run --bin workload_processor -- .

# Mathematical Atlas and Performance Coverage Testing

# Build standalone tools
build-tools:
	@echo "🔨 Building standalone mathematical atlas tools..."
	nix develop --command rustc -o simple_repository_mathematical_atlas src/bin/simple_repository_mathematical_atlas.rs
	nix develop --command rustc -o project_self_coverage_tile src/bin/project_self_coverage_tile.rs
	nix develop --command rustc -o project_performance_coverage_tile src/bin/project_performance_coverage_tile.rs
	nix develop --command rustc -o final_cli_tile_renderer src/bin/final_cli_tile_renderer.rs
	nix develop --command rustc -o final_standalone_test_runner src/bin/final_standalone_test_runner.rs
	@echo "✅ All tools built successfully!"

# Run mathematical atlas tools
test-atlas: build-tools
	@echo "🔬 Running Mathematical Atlas Tests..."
	mkdir -p $(RESULTS_DIR)
	./simple_repository_mathematical_atlas > $(RESULTS_DIR)/atlas_output_$(DATE).txt 2>&1
	@echo "✅ Mathematical atlas tests complete"

# Run performance coverage tools
test-coverage: build-tools
	@echo "📊 Running Performance Coverage Tests..."
	mkdir -p $(RESULTS_DIR)
	./project_self_coverage_tile > $(RESULTS_DIR)/self_coverage_$(DATE).txt 2>&1
	./project_performance_coverage_tile > $(RESULTS_DIR)/performance_coverage_$(DATE).txt 2>&1
	@echo "✅ Performance coverage tests complete"

# Run test suite
test-suite: build-tools
	@echo "🧪 Running Test Suite..."
	mkdir -p $(RESULTS_DIR)
	./final_standalone_test_runner > $(RESULTS_DIR)/test_suite_$(DATE).txt 2>&1
	@echo "✅ Test suite complete"

# Run all tests and save results
test-all: test-atlas test-coverage test-suite
	@echo "📋 Generating comprehensive test report..."
	mkdir -p $(RESULTS_DIR)
	echo "# Mathematical Atlas & Performance Coverage System - Test Results" > $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "Generated on: $(DATE)" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "## Test Results Summary" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "### Mathematical Atlas Tests" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	cat $(RESULTS_DIR)/atlas_output_$(DATE).txt >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "### Performance Coverage Tests" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	cat $(RESULTS_DIR)/self_coverage_$(DATE).txt >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	cat $(RESULTS_DIR)/performance_coverage_$(DATE).txt >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "### Test Suite Results" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	cat $(RESULTS_DIR)/test_suite_$(DATE).txt >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "## Generated Files" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "```" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	ls -la repository_atlas_output/ >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "```" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "## Tool Binaries" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "```" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	ls -la simple_repository_mathematical_atlas project_self_coverage_tile project_performance_coverage_tile final_cli_tile_renderer final_standalone_test_runner >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	echo "```" >> $(RESULTS_DIR)/comprehensive_report_$(DATE).md
	@echo "✅ Comprehensive test report generated: $(RESULTS_DIR)/comprehensive_report_$(DATE).md"

# Upload results to pastebinit
upload-results: test-all
	@echo "📤 Uploading test results to pastebinit..."
	@echo "============================================"
	@echo ""
	@echo "📋 Uploading comprehensive report..."
	if command -v pastebinit >/dev/null 2>&1; then \
		pastebinit -i $(RESULTS_DIR)/comprehensive_report_$(DATE).md -f md; \
	else \
		echo "pastebinit not found. Installing..."; \
		sudo apt-get install pastebinit -y; \
		pastebinit -i $(RESULTS_DIR)/comprehensive_report_$(DATE).md -f md; \
	fi
	@echo ""
	@echo "📤 Uploading mathematical atlas output..."
	if command -v pastebinit >/dev/null 2>&1; then \
		pastebinit -i $(RESULTS_DIR)/atlas_output_$(DATE).txt -f text; \
	else \
		pastebinit -i $(RESULTS_DIR)/atlas_output_$(DATE).txt -f text; \
	fi
	@echo ""
	@echo "📤 Uploading self-coverage analysis..."
	if command -v pastebinit >/dev/null 2>&1; then \
		pastebinit -i $(RESULTS_DIR)/self_coverage_$(DATE).txt -f text; \
	else \
		pastebinit -i $(RESULTS_DIR)/self_coverage_$(DATE).txt -f text; \
	fi
	@echo ""
	@echo "📤 Uploading performance coverage analysis..."
	if command -v pastebinit >/dev/null 2>&1; then \
		pastebinit -i $(RESULTS_DIR)/performance_coverage_$(DATE).txt -f text; \
	else \
		pastebinit -i $(RESULTS_DIR)/performance_coverage_$(DATE).txt -f text; \
	fi
	@echo ""
	@echo "📤 Uploading test suite results..."
	if command -v pastebinit >/dev/null 2>&1; then \
		pastebinit -i $(RESULTS_DIR)/test_suite_$(DATE).txt -f text; \
	else \
		pastebinit -i $(RESULTS_DIR)/test_suite_$(DATE).txt -f text; \
	fi
	@echo ""
	@echo "✅ All results uploaded to pastebinit!"
	@echo "📁 Results saved in: $(RESULTS_DIR)/"
	@echo "📄 Comprehensive report: $(RESULTS_DIR)/comprehensive_report_$(DATE).md"
