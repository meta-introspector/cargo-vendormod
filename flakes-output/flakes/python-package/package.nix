{ pkgs, lib, src ? ./., buildPythonPackage, ... }:

buildPythonPackage {
  pname = "python-package";
  version = "0.1.0";
  inherit src;
  format = "setuptools";
  pyproject = true;

  nativeBuildInputs = with pkgs; [ setuptools ];

    # default build/install phases

  doCheck = false;
}
