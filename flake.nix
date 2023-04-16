{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
              qt5.qtbase
              qt5.qtdeclarative
              qt5.qtquickcontrols2
              qt5.qmake
              plasma5Packages.qqc2-desktop-style
              plasma5Packages.kirigami2
              pkg-config
            ];

            shellHook = ''
            export QT_LIBRARY_PATH="${qt5.qtbase}/lib"
            export QT_INCLUDE_PATH="${qt5.qtbase.dev}/include"
            export QML2_IMPORT_PATH=${qt5.qtdeclarative.bin}/${qt5.qtbase.qtQmlPrefix}:${qt5.qtquickcontrols2.bin}/${qt5.qtbase.qtQmlPrefix}:${plasma5Packages.qqc2-desktop-style.bin}/${qt5.qtbase.qtQmlPrefix}:${plasma5Packages.kirigami2}/${qt5.qtbase.qtQmlPrefix}
            '';
          };
        });
}
