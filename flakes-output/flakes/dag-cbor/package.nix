{ pkgs, lib, src ? ./., ... }:

pkgs.stdenv.mkDerivation {
  pname = "dag-cbor";
  version = "0.1.0";
  inherit src;

  nativeBuildInputs = with pkgs; [ jdk maven ];

    buildPhase = ''
      mvn package -DskipTests -Dmaven.repo.local=$TMPDIR/repository
    '';
    installPhase = ''
      mkdir -p $out/share/java
      cp target/*.jar $out/share/java/
    '';

  doCheck = false;
}
