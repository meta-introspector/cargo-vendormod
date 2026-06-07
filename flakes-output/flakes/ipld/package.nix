{ pkgs, lib, src ? ./., ... }:

pkgs.buildNpmPackage {
  pname = "ipld";
  version = "0.1.0";
  inherit src;
  npmDeps = pkgs.fetchNpmDeps {
    inherit src;
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";  # set after first build
  };
  npmBuildScript = "build";
  npmBuildInputs = with pkgs; [ ];
  dontNpmBuild = false;
  doCheck = false;
}
