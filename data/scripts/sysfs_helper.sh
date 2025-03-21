#!/bin/bash

set -e

pci_blacklist_file_path="/etc/cfhdb/pci_blacklist"
usb_blacklist_file_path="/etc/cfhdb/usb_blacklist"
sysfs_remove_history="/tmp/cfhdb_sysfs_remove_history"

if [[ "$2" == "pci" ]]
then
  blacklist_file_path=$pci_blacklist_file_path
else
  blacklist_file_path=$usb_blacklist_file_path
fi

start_device () {
  if [ -f "$sysfs_remove_history" ]
  then
    DRIVER_NAME=$(grep "^$2 " "$sysfs_remove_history" | awk '{print $2}')
    if [ -z "$DRIVER_NAME" ]; then
      echo "$2" > /sys/bus/"$1"/drivers/"$3"/bind
      exit 1
    else
      echo "$2" > /sys/bus/"$1"/drivers/"$DRIVER_NAME"/bind
      sed -i "/^$2 /d" "$sysfs_remove_history"
    fi
  else
    exit 1
  fi
}

stop_device () {
  DRIVER_NAME=$(basename $(readlink "/sys/bus/"$1"/devices/"$2"/driver"))
  if [ -z "$DRIVER_NAME" ]; then
    echo "No driver found for device $1."
    exit 1
  fi
  TEMP_TEXTLINE="$2 $DRIVER_NAME"
  if [ -f "$sysfs_remove_history" ]
  then
    if grep -Fxq "$TEMP_TEXTLINE" "$sysfs_remove_history"
    then
      true
    else
      echo "$TEMP_TEXTLINE" >> "$sysfs_remove_history"
    fi
  else
    touch "$sysfs_remove_history"
    echo "$TEMP_TEXTLINE" >> "$sysfs_remove_history"
  fi
  echo "$2" > /sys/bus/"$1"/devices/"$2"/driver/unbind
}

enable_device () {
  if [ -f "$blacklist_file_path" ]
  then
     sed -i "/^${2}$/d" "$blacklist_file_path"
  fi
}

disable_device () {
  if [ -f "$blacklist_file_path" ]
  then
    if grep -Fxq "$2" "$blacklist_file_path"
    then
      true
    else
      echo "$2" >> "$blacklist_file_path"
    fi
  else
    touch "$blacklist_file_path"
    echo "$2" >> "$blacklist_file_path"
  fi
}

case "$1" in
    start_device)
        start_device "$2" "$3" "$4"
        ;;
    stop_device)
        stop_device "$2" "$3"
        ;;
    enable_device)
        enable_device "$2" "$3"
        ;;
    disable_device)
        disable_device "$2" "$3"
        ;;
esac