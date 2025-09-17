#! /bin/sh



# Icons for enabled/disabled states (customize icons)
ICON_ENABLED=""   # Example: Font Awesome check mark
ICON_DISABLED=""  # Example: Font Awesome cross


function list_history() {
  dunstctl history | jq '[.data.[] | .[] | {id: .id.data, summary: .summary.data, level: .level.data, body: .body.data, icon: .icon_path.data}]'
}

function toggle_notifications() {
  dunstctl set-paused toggle
  eww  update notification_state="$(print_icon)"
}

function print_icon() {
  current_state=$(dunstctl is-paused)
  if [ "$current_state" = "true" ]; then
    echo "$ICON_ENABLED"
  else
    echo "$ICON_DISABLED"
  fi
}

case "$1" in
  history)
    list_history
    ;;
  toggle)
    toggle_notifications
    ;;
  icon)
    print_icon
    ;;
  *)
    echo "Usage: $0 {history|toggle|icon}"
    exit 1
    ;;
esac

