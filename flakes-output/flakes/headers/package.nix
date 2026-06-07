{ pkgs, lib, src ? ./., ... }:

pkgs.stdenv.mkDerivation {
  pname = "headers";
  version = "0.1.0";
  inherit src;

  nativeBuildInputs = with pkgs; [ pkg-config ];
  buildInputs = with pkgs; [ ];

    buildPhase = ''
      make
    '';
    installPhase = ''
      make install
    '';

  doCheck = false;
}
