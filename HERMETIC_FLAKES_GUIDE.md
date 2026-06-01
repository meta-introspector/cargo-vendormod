# Guide: Creating Hermetic Flakes with Local Git Mirrors

This guide explains how to refactor a Nix flake to source its code from a local git mirror instead of a relative file path (`./.`). This approach ensures that builds are hermetic, reproducible, and immune to issues caused by a "dirty" git worktree or local-only changes.

## Goal

The primary goal is to make our Nix builds more robust by ensuring they are based on a specific, version-controlled commit from a git repository, rather than the current state of the filesystem.

- **Before:** `src = ./.;` (Depends on the current directory, can be "dirty")
- **After:** `src = self-src;` (Depends on an immutable, version-locked git commit)

## The Pattern

The pattern involves two main changes to your `flake.nix`:

1.  **Add a new input:** A new input is added to the `inputs` section. This input points to the project's own local git mirror.
2.  **Update the `src`:** The package definition inside `outputs` is changed to use the new input as its source.

---

## Step-by-Step Instructions

Here is the process to apply this pattern to an existing flake. We will use the `cargo-vendormod` project as an example.

### 1. Identify the Local Git Mirror URL

First, determine the URL of the local git mirror for your project. You can find this by running `git remote -v` inside the project's directory.

```sh
$ git remote -v
local   /home/mdupont/git/solana.solfunmeme.com/cargo-vendormod (fetch)
local   /home/mdupont/git/solana.solfunmeme.com/cargo-vendormod (push)
```

From this output, construct the `git+file://` URL:

`git+file:///home/mdupont/git/solana.solfunmeme.com/cargo-vendormod`

### 2. Modify `flake.nix`

You will now make three edits to your `flake.nix` file.

#### a. Add the Source Input

In the `inputs` section, add a new input for the project's own source. We'll name it `<project-name>-src`. For our example, this is `cargo-vendormod-src`. It should point to the URL you just constructed and include the `follows` for `nixpkgs` and `flake-utils` to ensure consistency.

**Before:**
```nix
  inputs = {
    nixpkgs.url = "git+file:///mnt/data1/git/github.com/NixOS/nixpkgs.git?ref=master";
    flake-utils.url = "git+file:///mnt/data1/git/github.com/numtide/flake-utils.git?ref=main";
    // ... other inputs
  };
```

**After:**
```nix
  inputs = {
    nixpkgs.url = "git+file:///mnt/data1/git/github.com/NixOS/nixpkgs.git?ref=master";
    flake-utils.url = "git+file:///mnt/data1/git/github.com/numtide/flake-utils.git?ref=main";
    // ... other inputs

    # Add this new input for the flake's own source
    cargo-vendormod-src = {
      url = "git+file:///home/mdupont/git/solana.solfunmeme.com/cargo-vendormod";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
```

#### b. Add the Input to `outputs`

The new input must be passed as an argument to the `outputs` function.

**Before:**
```nix
  outputs = { self, nixpkgs, flake-utils, crate2nix }:
```

**After:**
```nix
  outputs = { self, nixpkgs, flake-utils, crate2nix, cargo-vendormod-src }:
```

#### c. Update the Package `src`

Finally, find the package definition for your project and change its `src` attribute to point to the new input you added.

**Before:**
```nix
        my-package = pkgs.rustPlatform.buildRustPackage {
          pname = "my-package";
          version = "0.1.0";
          src = ./.; // This will be changed
          cargoLock.lockFile = ./Cargo.lock;
        };
```

**After:**
```nix
        my-package = pkgs.rustPlatform.buildRustPackage {
          pname = "my-package";
          version = "0.1.0";
          src = cargo-vendormod-src; // Use the new input
          cargoLock.lockFile = ./Cargo.lock;
        };
```

### 3. Update the Lock File

After saving your changes to `flake.nix`, you must update the `flake.lock` file to register the new input. You can do this by running any `nix` command that evaluates the flake, such as:

```sh
nix flake show
```

Or, more explicitly:

```sh
nix flake update
```

Nix will fetch the git repository from the local URL, select the latest commit, and record it in `flake.lock`. Your project will now build hermetically from that specific commit.
