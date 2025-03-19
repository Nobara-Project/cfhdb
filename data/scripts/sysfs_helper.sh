#!/bin/bash

set -e

blacklist_file_path="/etc/cfhdb/pci_blacklist"

start_device () {
  echo "$2" | sudo tee /sys/bus/"$1"/drivers/"$3"/bind
}

stop_device () {
  echo "$2" | sudo tee /sys/bus/"$1"/devices/"$2"/driver/unbind
}

enable_device () {
  if [ -f "$blacklist_file_path" ]
  then
     sed -i "/^${1}$/d" "$blacklist_file_path"
  fi
}

disable_device () {
  if [ -f "$blacklist_file_path" ]
  then
    if grep -Fxq "$1" "$blacklist_file_path"
    then
      true
    else
      echo "$1" >> "$blacklist_file_path"
    fi
  else
    touch "$blacklist_file_path"
    echo "$1" >> "$blacklist_file_path"
  fi
}

case "$1" in
    start_device)
        start_device "$2" "$3" "$4"
        ;;
    stop_device)
        stop_device "$2" "$3" "$4"
        ;;
    enable_device)
        enable_device "$2"
        ;;
    disable_device)
        disable_device "$2"
        ;;
esac