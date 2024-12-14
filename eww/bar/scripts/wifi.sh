#!/bin/sh

status=$(nmcli g | grep -oE "disconnected")
essid=$(nmcli c | grep wlp1s0 | awk -F "  " '{print ($1)}')

if [ $status ] ; then
    icon="󰖪"
    text=""
    color="#575268"

else
    icon=""
    text="${essid}"
    color="#a1bdce"
fi


if [[ "$1" == "color" ]]; then
    echo $color	
elif [[ "$1" == "text" ]]; then
	echo $text
elif [[ "$1" == "icon" ]]; then
	echo $icon
elif [[ "$1" == "toggle" ]]; then
	rfkill toggle wlan
fi
