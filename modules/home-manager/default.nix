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
    colors = lib.mkOption {
      type = lib.types.string;
      description = "A scss file to configure the color. Allows stylix integration via template.";
      example = ./colors.nix;
    };

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
      configDir = ./../../eww;
    };
    systemd.user.services = {
      glue = {
        Unit = {
          Description = "Glue Daemon Service";
          After = [ config.wayland.systemd.target ];
          PartOf = [ config.wayland.systemd.target ];
        };
        Service = {
          ExecStart = "${pkgs.glue}/bin/glue daemon";
          Restart = "always";
          RestartSec = "10s";
        };
        Install = {
          WantedBy = [ config.wayland.systemd.target ];
        };
      };
    };
  };
}
