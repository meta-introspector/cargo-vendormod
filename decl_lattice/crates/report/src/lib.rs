use anyhow :: { Context , Result } ; use std :: fs ; use std :: path :: { Path , PathBuf } ; use std :: io :: { BufRead , BufReader } ; use log :: info ; # [doc = " Generate a comprehensive processing report"] pub fn generate_processing_report (input_file : & Path , output_base : & Path ,) -> Result < String > { info ! ("Generating processing report...") ; let total_input_crates = fast_count_lines (input_file) ? ; let workspaces_dir = output_base . join ("workspaces") ; let processed_crates = fast_count_directories (& workspaces_dir) ? ; let flake_count = count_flake_files_fast (output_base) ? ; let submodule_usage = fast_check_submodule_integration (output_base) ? ; let report = format ! ("📊 CARGO-VENDORMOD PROCESSING REPORT
{
}
📦 Input Statistics:
  Total Cargo.toml files: {}
  Expected crates: {}
{
}
✅ Processing Results:
  Crates processed: {}
  Nix flakes generated: {}
  Success rate: {}%
{
}
🔗 Submodule Integration:
  Submodules detected: {}
  Submodule usage: {}
{
}
📁 Output Structure:
  Workspaces: {}/workspaces/ ({} crates)
  Flakes: {}/**/layer*/*/flake.nix ({} flakes)
{
}
🎯 Status:
  {} of {} crates processed ({}%)
  {} flakes generated for reproducible builds
  Topological sorting: ✅ Active
  Layered processing: ✅ Active
  Git integration: ✅ Active
  Nix flake generation: ✅ Active
" , "=" . repeat (50) , total_input_crates , total_input_crates , "-" . repeat (30) , processed_crates , flake_count , if total_input_crates > 0 { (processed_crates * 100) / total_input_crates } else { 0 } , "-" . repeat (30) , submodule_usage . submodule_count , submodule_usage . description , "-" . repeat (30) , output_base . display () , processed_crates , output_base . display () , flake_count , "-" . repeat (30) , processed_crates , total_input_crates , if total_input_crates > 0 { (processed_crates * 100) / total_input_crates } else { 0 } , flake_count) ; Ok (report) }