# ⚙️ Configuration File

Glue uses a TOML configuration file located at `~/.config/glue/config.toml`. The configuration supports environment variable overrides with the `GLUE_` prefix and follows a hierarchical structure.

## Configuration Structure

```toml
[general]
# General application settings

[battery]
# Battery monitoring and display settings

[coffee]
# Caffeine mode settings

[hyprland]
# Hyprland workspace settings

[event]
# Optional event handling configuration
```

## Default Configuration

The default configuration is automatically generated if no config file exists. You can generate a default config file with:

```sh
glue config generate
```

## Detailed Configuration Options

### General Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `log_level` | string | `"info"` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `eww_config` | string | `None` | Path to EWW configuration file |

**Example:**
```toml
[general]
log_level = "debug"
eww_config = "~/.config/eww/glue.yaml"
```

### Battery Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `charging_states` | array | `["", "", "", "", ""]` | Icons for battery charge levels |
| `full` | string | `"󱐥"` | Icon when battery is full |
| `charging` | string | `"󰂄"` | Icon when battery is charging |
| `empty` | string | `""` | Icon when battery is empty |
| `path` | string | `"/sys/class/power_supply/BAT0"` | Path to battery device |

**Example:**
```toml
[battery]
charging_states = ["🪫", "🔋", "🔌", "⚡", "💡"]
full = "🔋"
charging = "⚡"
path = "/sys/class/power_supply/BAT1"
```

### Coffee Mode Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `coffee` | string | `""` | Icon for active coffee mode |
| `relax` | string | `"󰒲"` | Icon for inactive coffee mode |
| `notification` | duration | `None` | Optional notification duration (e.g., "5min") |

**Example:**
```toml
[coffee]
coffee = "☕"
relax = "😴"
notification = "30min"
```

### Hyprland Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default_spaces` | integer | `5` | Number of default workspaces |

**Example:**
```toml
[hyprland]
default_spaces = 7
```

### Event Handling (Optional)

```toml
[event]
[[event.battery]]
charge = 20
state = "Discharging"
notify = "Low battery warning"
shell = "notify-send 'Low battery'"
hooks = ["/path/to/script1.sh", "/path/to/script2.sh"]
```

**Battery Event Fields:**
- `charge`: Battery percentage threshold (0-100)
- `state`: Battery state ("Charging", "Discharging", "Full")
- `notify`: Notification message
- `shell`: Shell command to execute
- `hooks`: List of scripts to run

## Environment Variables

All configuration options can be overridden using environment variables with the `GLUE_` prefix. For example:

```sh
export GLUE_GENERAL_LOG_LEVEL=debug
export GLUE_HYPRLAND_DEFAULT_SPACES=7
```

## Example Full Configuration

```toml
[general]
log_level = "debug"
eww_config = "~/.config/eww/glue.yaml"

[battery]
charging_states = ["🪫", "🔋", "🔌", "⚡", "💡"]
full = "🔋"
charging = "⚡"
empty = "🪫"
path = "/sys/class/power_supply/BAT0"

[coffee]
coffee = "☕"
relax = "😴"
notification = "30min"

[hyprland]
default_spaces = 7

[event]
[[event.battery]]
charge = 20
state = "Discharging"
notify = "Low battery warning"
shell = "notify-send 'Low battery'"
hooks = ["/path/to/low_battery.sh"]

[[event.battery]]
charge = 90
state = "Charging"
notify = "Battery almost full"
shell = "notify-send 'Battery almost full'"
```

## Notes

1. The configuration file is automatically created with default values on first run
2. Environment variables take precedence over file configuration
3. Command-line arguments override both file and environment configuration
