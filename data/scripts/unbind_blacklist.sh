#!/bin/bash

pci_blacklist_file_path="/etc/cfhdb/pci_blacklist"
usb_blacklist_file_path="/etc/cfhdb/usb_blacklist"
sysfs_remove_history="/tmp/cfhdb_sysfs_remove_history"

for device in $(cat $pci_blacklist_file_path)
do
  /usr/lib/cfhdb/scripts/sysfs_helper.sh stop_device pci $device
done

for device in $(cat $usb_blacklist_file_path)
do
  /usr/lib/cfhdb/scripts/sysfs_helper.sh stop_device usb $device
done
