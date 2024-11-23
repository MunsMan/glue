{
  description = "Rust base development flake for personal glue";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rust = pkgs.rust-bin.stable.latest.default;
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        with pkgs; {
          devShells.default = mkShell {
            packages = [ rust rust-analyzer nixpkgs-fmt ];
          };
          packages.default = rustPlatform.buildRustPackage {
            inherit (cargoToml.package) version name;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [ eww pkg-config ];
            packages = with pkgs; [ eww ];
          };
          formatter = pkgs.nixfmt-rfc-style;
        }) // {
      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.glue;
          tomlFormat = pkgs.formats.toml { };
        in
        {
          options.services.glue = {
            enable = lib.mkEnableOption "Glue service";

            settings = lib.mkOption {
              type = lib.types.submodule {
                options = {
                  battery = lib.mkOption {
                    type = lib.types.submodule {
                      options = {
                        chargingStates = lib.mkOption {
                          type = lib.types.listOf lib.types.str;
                          description = "List of charging states";
                        };
                        full = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing full battery";
                        };
                        charging = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing charging battery";
                        };
                        empty = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing empty battery";
                        };
                      };
                    };
                    description = "Battery configuration";
                  };

                  autostart = lib.mkOption {
                    type = lib.types.listOf lib.types.str;
                    default = [ ];
                    description = "List of programs to autostart";
                  };
                };
              };
              default = { };
              description = "Glue configuration settings";
            };
          };

          config = lib.mkIf cfg.enable {
            xdg.configFile."glue/config.toml".source = tomlFormat.generate "glue-config" cfg.settings;
            systemd.user.services.glue = {
              Unit = {
                Description = "Glue Daemon Service";
                After = [ "graphical-session.target" ];
                PartOf = [ "graphical-session.target" ];
              };
              Service = {
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/glue daemon";
                Restart = "always";
                RestartSec = "10s";
              };
              Install = {
                WantedBy = [ "graphical-session.target" ];
              };
            };
          };
        };
    };
}
