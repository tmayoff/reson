{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-utils,
    nixpkgs,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default;
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            openssl
          ];

          nativeBuildInputs = with pkgs; [
            (rust.override {extensions = ["rust-src" "llvm-tools-preview"];})
            rust-analyzer

            cargo-llvm-cov
          ];
        };
      }
    );
}
