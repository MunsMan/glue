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
nix run github:yourusername/my-system-utils
```

### Building from Source
```bash
git clone https://github.com/yourusername/my-system-utils.git
cd my-system-utils
cargo build --release
