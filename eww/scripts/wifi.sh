#!/bin/sh

# Enhanced WiFi component script for eww
# Provides icons, connection status, and TUI access via wezterm

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
    text="No WiFi"
    color="#575268"
    connected="false"
}

# Get WiFi status using NetworkManager
get_nm_status() {
    local status essid signal strength
    
    # Check overall connectivity
    status=$(nmcli g | tail -n1 | awk '{print $1}')
    
    # Get active WiFi connection
    essid=$(nmcli -t -f NAME,TYPE,DEVICE connection show --active | grep "wireless\|wifi" | head -1 | cut -d: -f1)
    
    if [ "$status" = "disconnected" ] || [ -z "$essid" ]; then
        icon="󰖪"
        text="Disconnected"
        color="#575268"
        connected="false"
    else
        # Get signal strength
        signal=$(nmcli -t -f SIGNAL device wifi list | head -1 | cut -d: -f1)
        
        # Set icon based on signal strength
        if [ -n "$signal" ] && [ "$signal" -gt 75 ]; then
            icon="󰤨"  # Excellent signal
        elif [ -n "$signal" ] && [ "$signal" -gt 50 ]; then
            icon="󰤥"  # Good signal
        elif [ -n "$signal" ] && [ "$signal" -gt 25 ]; then
            icon="󰤢"  # Fair signal
        else
            icon="󰤟"  # Poor signal
        fi
        
        text="${essid}"
        color="#a1bdce"
        connected="true"
    fi
}

# Get WiFi status using iwctl
get_iwctl_status() {
    local interface station_status essid
    
    # Get the wireless interface name (strip ANSI codes)
    interface=$(iwctl device list | sed 's/\x1b\[[0-9;]*m//g' | awk 'NR>4 && /wl/ {print $1}' | head -n1)
    
    if [ -z "$interface" ]; then
        icon="󰖪"
        text="No interface"
        color="#575268"
        connected="false"
    else
        # Check if station is connected
        station_status=$(iwctl station "$interface" show | grep "State" | awk '{print $2}')
        
        if [ "$station_status" != "connected" ]; then
            icon="󰖪"
            text="Disconnected"
            color="#575268"
            connected="false"
        else
            # Get SSID from iwctl
            essid=$(iwctl station "$interface" show | grep "Connected network" | awk '{for(i=3;i<=NF;i++) printf "%s ", $i; print ""}' | sed 's/[[:space:]]*$//')
            
            if [ -z "$essid" ]; then
                icon="󰖪"
                text="Unknown"
                color="#575268"
                connected="false"
            else
                # Use a general connected icon for iwctl (signal strength harder to get)
                icon="󰤨"
                text="${essid}"
                color="#a1bdce"
                connected="true"
            fi
        fi
    fi
}

# Determine WiFi tool and get status
WIFI_TOOL=$(detect_wifi_tool)

case "$WIFI_TOOL" in
    "nm")
        get_nm_status
        ;;
    "iwctl")
        get_iwctl_status
        ;;
    *)
        set_no_wifi_defaults
        ;;
esac

# Launch WiFi TUI manager using wezterm
open_wifi_tui() {
    if [ "$WIFI_TOOL" = "nm" ]; then
        if command -v nmtui > /dev/null 2>&1; then
            wezterm start --class "wifi-tui" -- nmtui
        else
            notify-send "WiFi Manager" "nmtui not available" -i network-wireless
        fi
    elif [ "$WIFI_TOOL" = "iwctl" ]; then
        if command -v impala > /dev/null 2>&1; then
            wezterm start --class "wifi-tui" -- impala
        else
            # Fallback to iwctl interactive mode
            wezterm start --class "wifi-tui" -- iwctl
        fi
    else
        notify-send "WiFi Manager" "No WiFi management tool available" -i network-wireless-offline
    fi
}

# Toggle WiFi radio
toggle_wifi() {
    if command -v rfkill > /dev/null 2>&1; then
        rfkill toggle wlan
        # Give a moment for the state to change
        sleep 1
        if [ "$WIFI_TOOL" = "nm" ]; then
            get_nm_status
        elif [ "$WIFI_TOOL" = "iwctl" ]; then
            get_iwctl_status
        fi
    else
        notify-send "WiFi Toggle" "rfkill not available" -i network-wireless
    fi
}

# Main command handler
case "$1" in
    "color")
        echo "$color"
        ;;
    "text")
        echo "$text"
        ;;
    "icon")
        echo "$icon"
        ;;
    "connected")
        echo "$connected"
        ;;
    "tool")
        echo "$WIFI_TOOL"
        ;;
    "toggle")
        toggle_wifi
        ;;
    "open"|"tui")
        open_wifi_tui
        ;;
    "status")
        echo "Tool: $WIFI_TOOL, Connected: $connected, SSID: $text"
        ;;
    *)
        echo "Usage: $0 {color|text|icon|connected|tool|toggle|open|tui|status}"
        exit 1
        ;;
esac