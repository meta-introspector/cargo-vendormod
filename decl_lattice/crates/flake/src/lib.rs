use anyhow :: { Context , Result } ; use cargo_vendormod :: config :: Config ; use clap :: { Parser , Subcommand } ; use std :: path :: PathBuf ; use std :: fs ; use cargo_vendormod :: global_dep_graph :: GlobalDependencyGraphBuilder ; use rayon :: prelude :: * ; use cargo_vendormod :: global_dep_graph :: GlobalDependencyGraphBuilder ; fn generate_flake (_crate_dir : & PathBuf , crate_name : & str , output_dir : & PathBuf) -> Result < () > { let flake_dir = output_dir . join ("flakes") . join (crate_name) ; fs :: create_dir_all (& flake_dir) ? ; let flake_content = format ! (r#"
{{
  description = "{}";

  outputs = {{ self, nixpkgs }}: {{
    packages.x86_64-linux.default = nixpkgs.mkShell {{
      buildInputs = with nixpkgs; [
        rustc
        cargo
      ];
    }};
  }};
}}
"# , crate_name) ; fs :: write (flake_dir . join ("flake.nix") , flake_content) ? ; Ok (()) }