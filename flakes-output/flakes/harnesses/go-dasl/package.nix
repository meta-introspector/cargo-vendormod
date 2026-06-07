{ pkgs, lib, src ? ./., ... }:

pkgs.buildGoModule {
  pname = "harnesses/go-dasl";
  version = "0.1.0";
  inherit src;
  vendorHash = null;   # set to "sha256-..." after first build
  proxyVendor = true;  # fetch from Go proxy instead of vendoring
  doCheck = false;
}
