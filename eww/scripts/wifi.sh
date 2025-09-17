#!/bin/sh

# Detect which WiFi tool is available
detect_wifi_tool() {
    if command -v nmcli > /dev/null 2>&1; then
        echo "nm"
    elif command -v iwctl > /dev/null 2>&1; then
        echo "iwctl"
    else
        echo "none"
    fi
}

# Set default values for no WiFi tool
set_no_wifi_defaults() {
    icon="󰖪"
    text="No WiFi tool"
    color="#575268"
}

# Get WiFi status using NetworkManager
get_nm_status() {
    local status essid
    status=$(nmcli g | grep -oE "disconnected")
    essid=$(nmcli c | grep wlp1s0 | awk -F "  " '{print ($1)}')
    
    if [ "$status" ]; then
        icon="󰖪"
        text=""
        color="#575268"
    else
        icon=""
        text="${essid}"
        color="#a1bdce"
    fi
}

# Get WiFi status using iwctl
get_iwctl_status() {
    local interface station_status essid
    # Get the wireless interface name
    interface=$(iwctl device list | awk 'NR>4 && /wlan/ {print $1}' | head -n1)
    
    if [ -z "$interface" ]; then
        icon="󰖪"
        text=""
        color="#575268"
    else
        # Check if station is connected
        station_status=$(iwctl station "$interface" show | grep "State" | awk '{print $2}')
        
        if [ "$station_status" != "connected" ]; then
            icon="󰖪"
            text=""
            color="#575268"
        else
            # Get SSID from iwctl
            essid=$(iwctl station "$interface" show | grep "Connected network" | awk '{for(i=3;i<=NF;i++) printf "%s ", $i; print ""}' | sed 's/[[:space:]]*$//')
            
            if [ -z "$essid" ]; then
                icon="󰖪"
                text=""
                color="#575268"
            else
                icon=""
                text="${essid}"
                color="#a1bdce"
            fi
        fi
    fi
}

WIFI_TOOL=$(detect_wifi_tool)

# Get WiFi status based on available tool
if [ "$WIFI_TOOL" = "nm" ]; then
    get_nm_status
elif [ "$WIFI_TOOL" = "iwctl" ]; then
    get_iwctl_status
else
    set_no_wifi_defaults
fi

# Function to detect available terminal
detect_terminal() {
    for term in "wezterm" "$TERMINAL" "alacritty" "kitty" "foot" "gnome-terminal" "xterm"; do
        if command -v "$term" > /dev/null 2>&1; then
            echo "$term"
            return
        fi
    done
    echo "wezterm"  # fallback
}

# Launch terminal with appropriate WiFi management tool
launch_terminal_with_app() {
    local terminal="$1"
    local app="$2"
    
    case "$terminal" in
        "wezterm")
            wezterm start "$app"
            ;;
        "alacritty"|"kitty"|"foot")
            "$terminal" -e "$app"
            ;;
        "gnome-terminal")
            gnome-terminal -- "$app"
            ;;
        *)
            "$terminal" -e "$app"
            ;;
    esac
}

# Open WiFi manager based on available tool
open_wifi_manager() {
    local terminal app
    terminal=$(detect_terminal)
    
    if [ "$WIFI_TOOL" = "nm" ]; then
        app="nmtui"
    elif [ "$WIFI_TOOL" = "iwctl" ]; then
        app="impala"
    else
        return 1
    fi
    
    launch_terminal_with_app "$terminal" "$app"
}

if [[ "$1" == "color" ]]; then
    echo $color	
elif [[ "$1" == "text" ]]; then
	echo $text
elif [[ "$1" == "icon" ]]; then
	echo $icon
elif [[ "$1" == "toggle" ]]; then
	rfkill toggle wlan
elif [[ "$1" == "tool" ]]; then
	echo $WIFI_TOOL
elif [[ "$1" == "open" ]]; then
    open_wifi_manager
fi
