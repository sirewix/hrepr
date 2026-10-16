{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        { system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };
          toolchain = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [
              "rustfmt"
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];
          };
        in
        {
          devShells.default = pkgs.mkShell {
            packages = [ toolchain pkgs.prek ];
          };
        };
    };
}
