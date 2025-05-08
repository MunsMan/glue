#!/bin/sh

bat=/sys/class/power_supply/BAT0/
per="$(cat "$bat/capacity")"

toggle() {
    state=$(eww state | rg battery_percent | cut -d " " -f 2)
    if [ "$state" = "true" ]; then
        eww update battery_percent=false
    else
        eww update battery_percent=true
    fi
}

[ "$1" = "toggle" ] && toggle && exit
exit
