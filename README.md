# Glue: Seamless System Utilities Integration

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-%23dea584.svg?logo=rust&logoColor=white)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?logo=nixos&logoColor=white)
![Home Manager](https://img.shields.io/badge/Home_Manager-41439A?logo=home-assistant&logoColor=white)

**Glue** is a sophisticated collection of system utilities designed to create perfect harmony between Eww, Hyprland, and other components of your desktop environment.
Built with Rust for performance and reliability, Glue offers both a command-line interface and a daemon service for comprehensive system integration.

![Status bar example](./docs/eww.jpeg)

## 🌟 Key Features

- **Unified System Control**: Seamlessly connect `eww`, `Hyprland`, and other desktop components
- **Dual Operation Modes**:
  - **CLI Interface**: Powerful CLI tools for direct system interaction
  - **Daemon Service**: Background process listening for and responding to system events
- **NixOS Integration**: Fully compatible with NixOS and Home Manager
- **Performance Optimized**: Rust implementation ensures speed and memory safety


## 🚀 Getting Started

Both the [CLI](./docs/cli.md) and the [Config File](./docs/config.md) are documented under `./docs`.

### NixOS/Home Manager Installation (Recommended)

Glue is designed to work perfectly with NixOS and Home Manager.
The complete setup is available as a Home Manager module, making installation and configuration effortless:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    glue.url = "github:munsman/glue";
  };

  outputs = { nixpkgs, home-manager, glue, ... }: {
    homeConfigurations."user" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;

      modules = [
        {
          imports = [ inputs.glue.homeManagerModules.default ];
          services.glue = {
            enable = true;
            settings = {
              coffee = {
                notification = "30m";
              };
              event = {
                battery = [
                  {
                    charge = 5;
                    state = "Discharging";
                    notify = "Battery Low: 5%";
                  }
                ];
              };
            };
          };
        }
      ];
    };
  };
}

```

### Quick Trail with Nix Flakes

For a quick test run it without a full installation:

```bash
nix run github:munsman/glue
```

### Building from Source

For development or non-Nix systems:

```sh
git clone https://github.com/munsman/glue.git
cd glue
cargo build --release
```
## 🔮 Future Development

Our roadmap includes:
- Complete self-containment within the Nix ecosystem
- Enhanced Home Manager module with more configuration options
- Expanded compatibility with additional window managers and system components
- More comprehensive event handling capabilities

## 🔧 Dependencies

Glue is designed to work with:
  - [EWW](https://github.com/elkowar/eww) - System information and control widgets
  - [Hyprland](https://hypr.land/) - Dynamic tiling Wayland compositor

## 🤝 Contributing

Contributions are welcome!
Please feel free to submit issues or pull requests to help improve Glue.
