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
            vscode-css-languageserver
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
          ];
          packages = with pkgs; [ eww ];
        };

        checks = {
          wifi-script-tests = pkgs.stdenv.mkDerivation {
            name = "wifi-script-tests";
            src = ./.;
            buildInputs = [ python3 ];
            buildPhase = ''
              # Copy test files
              mkdir -p $out/tests
              cp eww/scripts/wifi.sh $out/tests/
              cp eww/scripts/test_wifi.py $out/tests/
            '';
            checkPhase = ''
              cd $out/tests
              chmod +x wifi.sh test_wifi.py
              python3 test_wifi.py
            '';
            doCheck = true;
            installPhase = ''
              echo "Tests completed successfully" > $out/test-results
            '';
          };
        };

        apps = {
          test-wifi = {
            type = "app";
            program = "${pkgs.writeShellScript "test-wifi" ''
              cd ${./.}/eww/scripts
              ${python3}/bin/python3 test_wifi.py
            ''}";
            meta = {
              description = "Run WiFi script tests";
              mainProgram = "test-wifi";
            };
          };
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    ) // {
      # Home Manager modules (legacy output name - still supported)
      homeManagerModules = {
        glue = ./modules/home-manager;
        default = self.homeManagerModules.glue;
      };
      
      # Standard flake output name
      homeModules = self.homeManagerModules;
    };
}
