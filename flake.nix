{
  description = "Rust base development flake for personal glue";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            glue = self.packages.default.glue;
          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };
        rust = pkgs.rust-bin.stable.latest.default;
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            rust
            rust-analyzer
            nixfmt-rfc-style
            pkg-config
            libdbusmenu
            dbus
            nixd
            jq
            dunst
            eww
            wezterm
            vscode-css-languageserver
            bash-language-server
            python3
            claude-code
          ];
        };
        packages.default = rustPlatform.buildRustPackage {
          inherit (cargoToml.package) version name;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [
            pkg-config
            dbus
            wrapGAppsHook
          ];
          buildInputs = with pkgs; [
            eww
            wezterm
            impala
            libnotify
          ];
          postInstall = ''
            # Ensure wezterm is available in PATH for the wifi script
            wrapProgram $out/bin/glue \
              --prefix PATH : ${lib.makeBinPath [ wezterm ]}
          '';
        };

        checks = import ./tests.nix { inherit pkgs cargoToml; };

        formatter = pkgs.nixfmt-rfc-style;
      }
    )
    // {
      # Home Manager modules (legacy output name - still supported)
      homeManagerModules = {
        glue = ./modules/home-manager;
        default = self.homeManagerModules.glue;
      };

      # Standard flake output name
      homeModules = self.homeManagerModules;
    };
}
