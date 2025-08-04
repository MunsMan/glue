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
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
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
            eww
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
        formatter = pkgs.nixfmt-rfc-style;
      }
    )
    // {
      homeManagerModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.services.glue;
          tomlFormat = pkgs.formats.toml { };
        in
        {
          options.services.glue = {
            enable = lib.mkEnableOption "Glue service";

            settings = lib.mkOption {
              type = lib.types.submodule {
                freeformType = tomlFormat.type;
                options = {
                  battery = lib.mkOption {
                    type = lib.types.submodule {
                      options = {
                        chargingStates = lib.mkOption {
                          type = lib.types.listOf lib.types.str;
                          description = "List of charging states";
                          default = [
                            ""
                            ""
                            ""
                            ""
                            ""
                          ];
                        };
                        full = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing full battery";
                          default = "󱐥";
                        };
                        charging = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing charging battery";
                          default = "󰂄";
                        };
                        empty = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing empty battery";
                          default = "";
                        };
                      };
                    };
                    description = "Battery configuration";
                    default = { };
                  };

                  autostart = lib.mkOption {
                    type = lib.types.listOf lib.types.str;
                    description = "List of programs to autostart";
                    default = [ ];
                    example = [ "${pkgs._1password-gui}/bin/1password --silent" ];
                  };

                  coffee = lib.mkOption {
                    type = lib.types.submodule {
                      options = {
                        coffee = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing the coffeinated state";
                          default = "";
                        };
                        relax = lib.mkOption {
                          type = lib.types.str;
                          description = "Character representing the decoffeinated state";
                          default = "󰒲";
                        };
                        notification = lib.mkOption {
                          type = lib.types.nullOr lib.types.str;
                          description = "Character representing the decoffeinated state";
                          example = "1h";
                          default = null;
                        };
                      };
                    };
                    description = "Coffee configuration";
                    default = { };
                  };
                };
              };
              default = null;
              description = "Glue configuration settings";
            };
          };

          config = lib.mkIf cfg.enable {
            xdg.configFile."glue/config.toml".source = tomlFormat.generate "glue-config" (
              lib.attrsets.filterAttrsRecursive (_key: value: value != null) cfg.settings
            );
            programs.eww = {
              enable = true;
              configDir = ./eww/bar;
            };
            systemd.user.services = {
              glue = {
                Unit = {
                  Description = "Glue Daemon Service";
                  After = [ "eww-daemon.service" ];
                  Requires = [ "eww-daemon.service" ];
                };
                Service = {
                  ExecStart = "${self.packages.${pkgs.system}.default}/bin/glue daemon";
                  Restart = "always";
                  RestartSec = "10s";
                };
                Install = {
                  WantedBy = [ config.wayland.systemd.target ];
                };
              };
              eww-daemon = {
                Unit = {
                  Description = "Eww Daemon";
                  PartOf = [ config.wayland.systemd.target ];
                  After = [ config.wayland.systemd.target ];
                  ConditionEnvironment = "WAYLAND_DISPLAY";
                };
                Service = {
                  ExecStart = "${pkgs.eww}/bin/eww daemon";
                  Restart = "on-failure";
                  RestartSec = 1;
                };
                Install = {
                  WantedBy = [ config.wayland.systemd.target ];
                };
              };
              bar = {
                Unit = {
                  Description = "Eww Status Bar";
                  After = [ "eww-daemon.service" ];
                  Requires = [ "eww-daemon.service" ];
                  ConditionEnvironment = "WAYLAND_DISPLAY";
                };
                Service = {
                  ExecStart = "${pkgs.eww}/bin/eww open bar";
                  Restart = "on-failure";
                  RestartSec = 1;
                };
                Install = {
                  WantedBy = [ config.wayland.systemd.target ];
                };
              };
            };
          };
        };
    };
}
