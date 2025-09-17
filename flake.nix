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
          ];
          postInstall = ''
            # Ensure wezterm is available in PATH for the wifi script
            wrapProgram $out/bin/glue \
              --prefix PATH : ${lib.makeBinPath [ wezterm ]}
          '';
        };

        checks = import ./tests.nix { inherit pkgs rust; };

        apps = {
          test-all = {
            type = "app";
            program = "${pkgs.writeShellScript "test-all" ''
              echo "🧪 Running all glue tests..."
              echo "=============================="
              
              echo "🦀 Running Rust tests..."
              ${rust}/bin/cargo test
              
              echo ""
              echo "📜 Running script tests..."
              cd ${./.}/eww/scripts
              ${python3}/bin/python3 test_wifi.py
              
              echo ""
              echo "✅ All tests completed!"
            ''}";
            meta = {
              description = "Run all tests (Rust + scripts)";
              mainProgram = "test-all";
            };
          };
          
          test-rust = {
            type = "app";
            program = "${pkgs.writeShellScript "test-rust" ''
              echo "🦀 Running Rust tests..."
              ${rust}/bin/cargo test
            ''}";
            meta = {
              description = "Run only Rust tests";
              mainProgram = "test-rust";
            };
          };
          
          test-scripts = {
            type = "app";
            program = "${pkgs.writeShellScript "test-scripts" ''
              echo "📜 Running script tests..."
              cd ${./.}/eww/scripts
              ${python3}/bin/python3 test_wifi.py
            ''}";
            meta = {
              description = "Run only script tests";
              mainProgram = "test-scripts";
            };
          };
        };

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
