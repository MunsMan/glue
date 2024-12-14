#!/bin/sh

blocked=$(rfkill list bluetooth | rg -oi "Soft blocked: yes")
devices=$(bluetoothctl info | rg Name | xargs)


if [[ $blocked ]] ; then
    icon="󰂲"
    text="bluetooth off"
    color="#575268"
else 
    icon="󰂯"
    text="No devices connected!"
    color="#a1bdce"
    if [ "$devices" ]; then
        icon="󰂱"
        text="$devices"
        color="#a1bdce"
    fi
fi



if [[ "$1" == "color" ]]; then
    echo $color	
elif [[ "$1" == "tooltip" ]]; then
	echo $text
elif [[ "$1" == "icon" ]]; then
	echo $icon
elif [[ "$1" == "toggle" ]]; then
    if [[ $blocked ]]; then
        eww update bluetooth-icon="$icon" 
        eww update bluetooth-color="$color" 
        rfkill unblock bluetooth
    else
        eww update bluetooth-icon="$icon" 
        eww update bluetooth-color="$color" 
        rfkill block bluetooth
    fi
fi
