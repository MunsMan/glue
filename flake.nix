{
  description = "Rust base development flake for personal glue";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      with pkgs; {
        devShells.default = mkShell {
          packages = [ rust rust-analyzer ];
        };
        packages.default = rustPlatform.buildRustPackage {
                        inherit (cargoToml.package) version name;
                        src = ./.;
                        cargoLock.lockFile = ./Cargo.lock;
                        nativeBuildInputs = with pkgs; [ eww pkg-config ];
                    };
      });
}
