### **Plan: Curate a Focused Test Suite from DASL Analysis**

The goal is to leverage the "compressed findings" from your shmem, binary analyses, and documentation to build a structured, cross-language test suite.

---

#### **Phase 1: Discovery & Inventory (What do we have?)**

First, we need to gather and understand the scattered testing and analysis assets.

1.  **Locate Pre-identified Test Cases:** Search for and analyze `TEST_CASES.txt` and the `extract_individual_tests.py` script mentioned in the documentation. This is our most direct path to a baseline set of tests.
2.  **Inventory Analysis Artifacts:** Systematically list the "compressed findings." This includes the `DMZ` analysis of `0xD8 0x2A`, the hypermorphism data, code-arrow graphs, and any failure witnesses stored in shmem or the filesystem.
3.  **Identify Existing Test Runners:** Find and document the scripts currently used to run tests (e.g., `run-all-tests`, `benchmark_*.sh`, Nix checks) to understand the existing execution flow.

---

#### **Phase 2: Analysis & Prioritization (Where should we focus?)**

With the inventory, we'll use the analysis to prioritize our efforts.

1.  **Map Test Coverage Gaps:** Use the analysis artifacts (like the code-review graph or scan results) to identify "critical" code sections that have low or no test coverage. The areas related to the `0xD8 0x2A` CID signature and other "spectrally interesting" code from the DMZ analysis are top priorities.
2.  **Define Cross-Language Test Vectors:** Use the findings on semantic equivalence to define a set of inputs (e.g., specific CBOR payloads) that should produce equivalent outputs or behaviors across the Rust, Go, Python, and JavaScript implementations.
3.  **Systematize Failure Cases:** Collect all known "failure witnesses." These are invaluable and will be converted into a formal regression test suite to ensure bugs stay fixed.

---

#### **Phase 3: Extraction & Generation (Build the test assets)**

Now we'll create the actual test assets in a structured way.

1.  **Create a Unified Test Case Format:** Define a simple, language-agnostic format (like JSON) to store our curated tests. Each test case will include an ID, a description, input data, and expected outcomes (output, success/fail status, error messages).
2.  **Extract Existing Tests:** Run the `extract_individual_tests.py` script (if found) and convert its output into our new unified JSON format. Do the same for the contents of `TEST_CASES.txt` and the failure witnesses.
3.  **Generate New Tests for Gaps:** For the high-priority, untested code identified in Phase 2, we will write new test cases in our JSON format, focusing on edge cases and core functionality.

---

#### **Phase 4: Organization & Execution (Create the test suite)**

Finally, we'll assemble everything into a coherent and runnable suite.

1.  **Structure the Curated Test Suite:** Create a new, clean directory (e.g., `dasl_curated_tests/`). Inside, we will organize the JSON test files by type: `cross_language/`, `regression/` (from failure witnesses), and `coverage/` (for new tests).
2.  **Develop a Unified Test Runner:** Create a master script that:
    *   Parses the JSON test case files.
    *   Invokes the correct implementation (Rust, Go, etc.) with the specified input.
    *   Compares the actual result against the expected outcome.
    *   Generates a clear report of passes and failures.
3.  **Integrate with Nix:** Package the new test suite and runner into the project's `flake.nix`. This will make the entire suite easily and reproducibly runnable with a single command like `nix flake check .#dasl-curated-tests`.
