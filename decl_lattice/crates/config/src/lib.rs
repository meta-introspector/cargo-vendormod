use std :: path :: { Path , PathBuf } ; use serde :: { Deserialize , Serialize } ; use anyhow :: { Context , Result } ; use super :: * ; # [doc = " Generate a sample config file"] pub fn generate_sample_config () -> String { r#"# cargo-vendormod configuration
# Copy to ./vendormod.toml or ~/.config/cargo-vendormod/config.toml

# Git executable path
git-path = "git"

# Default directories (relative to workspace)
vendor-dir = "vendor"
submodules-dir = "submodules"

# Bare git mirrors directory (absolute path recommended)
# Example: ~/git or /home/user/git
mirrors-dir = "~/git"

# Default target branch for submodules
target-branch = "main"

# Version branch format (use {} as placeholder for version)
version-branch-format = "v{}"

# Whether to automatically create version branches
create-version-branches = false

# Default number of threads for parallel operations
default-threads = 8
"# . to_string () }