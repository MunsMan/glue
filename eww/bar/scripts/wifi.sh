#!/bin/sh

# Detect which WiFi tool is available
if command -v nmcli > /dev/null 2>&1; then
    WIFI_TOOL="nm"
elif command -v iwctl > /dev/null 2>&1; then
    WIFI_TOOL="iwctl"
else
    # Fallback if neither is available
    icon="󰖪"
    text="No WiFi tool"
    color="#575268"
    WIFI_TOOL="none"
fi

if [ "$WIFI_TOOL" = "nm" ]; then
    # NetworkManager implementation
    status=$(nmcli g | grep -oE "disconnected")
    essid=$(nmcli c | grep wlp1s0 | awk -F "  " '{print ($1)}')
    
    if [ $status ] ; then
        icon="󰖪"
        text=""
        color="#575268"
    else
        icon=""
        text="${essid}"
        color="#a1bdce"
    fi

elif [ "$WIFI_TOOL" = "iwctl" ]; then
    # iwctl implementation
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
fi

# Function to detect available terminal
detect_terminal() {
    for term in "$TERMINAL" "wezterm" "alacritty" "kitty" "foot" "gnome-terminal" "xterm"; do
        if command -v "$term" > /dev/null 2>&1; then
            echo "$term"
            return
        fi
    done
    echo "xterm"  # fallback
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
    terminal=$(detect_terminal)
    
    if [ "$WIFI_TOOL" = "nm" ]; then
        # Use nmtui for NetworkManager
        case "$terminal" in
            "wezterm")
                wezterm start nmtui
                ;;
            "alacritty"|"kitty"|"foot")
                $terminal -e nmtui
                ;;
            "gnome-terminal")
                gnome-terminal -- nmtui
                ;;
            *)
                $terminal -e nmtui
                ;;
        esac
    elif [ "$WIFI_TOOL" = "iwctl" ]; then
        # Use impala for iwctl
        case "$terminal" in
            "wezterm")
                wezterm start impala
                ;;
            "alacritty"|"kitty"|"foot")
                $terminal -e impala
                ;;
            "gnome-terminal")
                gnome-terminal -- impala
                ;;
            *)
                $terminal -e impala
                ;;
        esac
    fi
fi