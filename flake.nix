{
  description = "Flake utils demo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        boost = pkgs.boost186;
      in {
        devShell = pkgs.mkShell.override {stdenv = pkgs.clangStdenv;} {
          nativeBuildInputs = with pkgs; [
            clang-tools

            just
            meson
            mesonlsp
            muon
            ninja

            lcov
          ];

          BOOST_INCLUDEDIR = "${pkgs.lib.getDev boost}/include";
          BOOST_LIBRARYDIR = "${pkgs.lib.getLib boost}/lib";

          buildInputs = [
            boost
          ];
        };
      }
    );
}
