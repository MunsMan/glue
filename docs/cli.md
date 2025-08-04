# CLI Documentation

Glue provides a comprehensive CLI for system control and automation.
The interface is built with `clap` for robust argument parsing and help generation.

## Basic Usage

```sh
glue [OPTIONS] <COMMAND>
```

## Available Commands

| Command | Description |
|---------|-------------|
| `daemon` | Run the background service with `eww` integration |
| `workspace` | Manage `Hyprland` workspaces |
| `audio` | Control audio devices |
| `mic` | Manage microphone settings |
| `battery` | Get battery information |
| `brightness` | Control display brightness |
| `start` | Start system services |
| `wake-up` | Wake up system components |
| `lock` | Lock the screen |
| `coffee` | Manage caffeine-related features |
| `test` | Test system notifications |

## Command Reference

### `daemon` Command

Run the background service that handles system events and `eww` integration:

```sh
glue daemon [OPTIONS]
```

**Options:**
- `-e, --eww-config` - Path to EWW configuration file

**Example:**
```sh
glue daemon --eww-config ~/.config/eww/glue.yaml
```

### `workspace` Command

Manage Hyprland workspaces:

```sh
glue workspace [OPTIONS] [SUBCOMMAND]
```

**Options:**
- `--default-spaces` - Number of default workspaces (default: 5)

**Subcommands:**
- `update` - Update workspace configuration

**Examples:**
```sh
# Set default workspaces to 7
glue workspace --default-spaces 7

# Update workspace configuration
glue workspace update --default-spaces 5
```

### `audio` Command

Control audio devices:

```sh
glue audio <SUBCOMMAND>
```

**Subcommands:**
- `set <percent>` - Set volume to specific percentage
- `get` - Get current volume level
- `mute` - Toggle mute
- `increase` - Increase volume
- `decrease` - Decrease volume

**Examples:**
```sh
glue audio set 75
glue audio increase
glue audio mute
```

### `mic` Command

Manage microphone settings:

```sh
glue mic <SUBCOMMAND>
```

**Subcommands:**
- `mute` - Toggle microphone mute
- `get` - Get microphone status

**Examples:**
```sh
glue mic mute
glue mic get
```

### `brightness` Command

Control display brightness:

```sh
glue brightness <SUBCOMMAND>
```

**Subcommands:**
- `get` - Get current brightness level
- `set <percent>` - Set brightness to specific percentage
- `increase` - Increase brightness
- `decrease` - Decrease brightness

**Examples:**
```sh
glue brightness set 50
glue brightness increase
glue brightness get
```

### `coffee` Command

Manage caffeine-related features (likely for screen dimming/brightness control):

```sh
glue coffee <SUBCOMMAND>
```

**Subcommands:**
- `drink` - Activate caffeine mode
- `relax` - Deactivate caffeine mode
- `toggle` - Toggle caffeine mode
- `get` - Get current caffeine status

**Examples:**
```sh
glue coffee drink
glue coffee toggle
```

### `test` Command

Test system notifications:

```sh
glue test notification <text>
```

**Subcommands:**
- `notification <text>` - Send a test notification

**Example:**
```sh
glue test notification "Hello from Glue!"
```

### System Commands

Simple system control commands:

```sh
# Start system services
glue start

# Wake up system components
glue wake-up [--eww-config <path>]

# Lock the screen
glue lock
```

## Shell Completion

Glue supports shell completion for better user experience:

```sh
# For bash
glue completions bash | sudo tee /etc/bash_completion.d/glue

# For zsh
glue completions zsh > ~/.zsh/completion/_glue

# For fish
glue completions fish > ~/.config/fish/completions/glue.fish
```

### Configuration

*mainly for debugging*

While Glue primarily uses command-line arguments, some behaviors can be configured through environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `GLUE_EWW_CONFIG` | Default `eww` config path | `~/.config/eww/glue.yaml` |
| `GLUE_DEFAULT_SPACES` | Default number of workspaces | `5` |

