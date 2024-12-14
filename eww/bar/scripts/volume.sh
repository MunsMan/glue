#!/bin/sh

# Get the volume of the default audio sink
volume_info=$(wpctl get-volume @DEFAULT_AUDIO_SINK@)

# Extract the numeric value and convert to percentage
volume_percent=$(echo $volume_info | awk '{print int($2 * 100)}')

# Check if the device is muted
if [[ "$volume_info" == *"[MUTED]"* ]]; then
    icon=""  # Muted icon
else
    # Select icon based on volume level
    if [ $volume_percent -eq 0 ]; then
        icon=""  # Volume off icon
    elif [ $volume_percent -lt 33 ]; then
        icon=""  # Low volume icon
    elif [ $volume_percent -lt 66 ]; then
        icon=""  # Medium volume icon
    else
        icon=""  # High volume icon
    fi
fi

# Print the icon and volume percentage
echo "$icon"
