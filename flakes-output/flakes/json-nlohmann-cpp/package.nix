{ pkgs, lib, src ? ./., ... }:

pkgs.stdenv.mkDerivation {
  pname = "json-nlohmann-cpp";
  version = "0.1.0";
  inherit src;

  nativeBuildInputs = with pkgs; [ cmake pkg-config ];
  buildInputs = with pkgs; [ ];

    buildPhase = ''
      cmake -DCMAKE_INSTALL_PREFIX=$out ..
      make
    '';
    installPhase = ''
      make install
    '';

  doCheck = false;
}
