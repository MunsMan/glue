# My System Utils

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-%23dea584.svg?logo=rust&logoColor=white)

## Overview

**My System Utils** is a collection of personal system utilities designed to enhance the integration between Eww, Hyprland, and other components of your desktop environment. Written in Rust, this package provides a command-line interface (CLI) and a daemon for listening to system events, offering a seamless and efficient user experience.

## Features

- **CLI Interface**: Easily interact with your system utilities via a robust command-line interface.
- **Daemon**: A background service that listens for events and triggers actions accordingly.
- **Integration**: Connects and integrates smoothly with Eww, Hyprland, and more.
- **Performance**: Built with Rust for speed and safety.

## Installation

### Using Nix Flakes

If you are using the Nix package manager, you can easily install the package using Nix flakes. First, ensure you have Nix installed and flakes enabled. Then, run the following command:

```bash
nix run github:yourusername/glue
```

### Building from Source

```sh
git clone https://github.com/yourusername/glue.git
cd glue
cargo build --release
```
## Future Plans

The goal for this repository is to become fully self-contained and compatible with both [NixOS](https://nixos.org/) and [Home Manager](https://github.com/nix-community/home-manager). To achieve seamless system integration, I plan to develop a dedicated Home Manager module.

### To-Do List
- [ ] Implement Battery Information Display
- [ ] Managing Autostart Programs
  - Restart after Sleep
- [ ] Develop Error Handling Interface
- [ ] Configuration File
- [ ] Create Home Manager Module

Feel free to adjust any part of it to better fit your style or specific needs!

## Dependencies

- [EWW](https://github.com/elkowar/eww)
- [Hyprland](https://hyprland.org/)
