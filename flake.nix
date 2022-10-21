{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in
        with pkgs; rec {
          devShells.default = mkShell {
            buildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
              qt6.qtbase
              qt6.qtdeclarative
              qt6.qmake
            ];

            QT_LIBRARY_PATH = "${qt6.qtbase}/lib";
            QT_INCLUDE_PATH = "${qt6.qtbase.dev}/include";
          };
        });
}
