use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CfhdbPciDevice {
    // String identification
    pub class_name: String,
    pub device_name: String,
    pub vendor_name: String,
    // Vendor IDs
    pub class_id: String,
    pub vendor_id: String,
    pub device_id: String,
    // System Info
    pub sysfs_busid: String,
    pub sysfs_id: String,
    pub kernel_driver: String,
    // Cfhdb Extras
    pub vendor_icon_name: String,
    pub available_profiles: Rc<RefCell<Option<Vec<CfhdbPciProfile>>>>,
}

impl CfhdbPciDevice {
    fn get_kernel_driver(busid: &str) -> Option<String> {
        let device_uevent_path = format!("/sys/bus/pci/devices/{}/uevent", busid);
        match std::fs::read_to_string(device_uevent_path) {
            Ok(content) => {
                for line in content.lines() {
                    if line.starts_with("DRIVER=") {
                        if let Some(value) = line.splitn(2, '=').nth(1) {
                            return Some(value.to_string());
                        }
                    }
                }
            }
            Err(_) => {
            }
        }
        return None;
    }

    pub fn set_available_profiles(profile_data: &[CfhdbPciProfile], device: &Self) {
        let mut available_profiles: Vec<CfhdbPciProfile> = vec![];
        for profile in profile_data.iter() {
            let matching = {
                if
                (profile.blacklisted_class_ids.contains(&"*".to_owned()) || profile.blacklisted_class_ids.contains(&device.class_id))
                ||
                (profile.blacklisted_vendor_ids.contains(&"*".to_owned()) || profile.blacklisted_vendor_ids.contains(&device.vendor_id))
                ||
                (profile.blacklisted_device_ids.contains(&"*".to_owned()) || profile.blacklisted_device_ids.contains(&device.device_id))
                {
                    false
                } else {
                    (profile.class_ids.contains(&"*".to_owned()) || profile.class_ids.contains(&device.class_id))
                    &&
                    (profile.vendor_ids.contains(&"*".to_owned()) || profile.vendor_ids.contains(&device.vendor_id))
                    &&
                    (profile.device_ids.contains(&"*".to_owned()) || profile.device_ids.contains(&device.device_id))
                }
            };
    
            if matching {
                available_profiles.push(profile.clone());
            };
    
            if !available_profiles.is_empty() {
                *device.available_profiles.borrow_mut() = Some(available_profiles.clone());
            };
        }
    }
    
    pub fn get_devices() -> Option<Vec<Self>> {
        let from_hex =
            |hex_number: u32, fill: usize| -> String { format!("{:01$x}", hex_number, fill) };
    
        // Initialize
        let mut pacc = libpci::PCIAccess::new(true);
    
        // Get hardware devices
        let pci_devices = pacc.devices()?;
        let mut devices = vec![];
    
        for mut iter in pci_devices.iter_mut() {
            // fill in header info we need
            iter.fill_info(libpci::Fill::IDENT as u32 | libpci::Fill::CLASS as u32);
    
            let item_class = iter.class()?;
            let item_vendor = iter.vendor()?;
            let item_device = iter.device()?;
            let item_class_id = from_hex(iter.class_id()? as _, 4);
            let item_device_id = from_hex(iter.device_id()? as _, 4);
            let item_vendor_id = from_hex(iter.vendor_id()? as _, 4);
            let item_sysfs_busid = format!(
                "{}:{}:{}.{}",
                from_hex(iter.domain()? as _, 4),
                from_hex(iter.bus()? as _, 2),
                from_hex(iter.dev()? as _, 2),
                iter.func()?,
            );
            let item_sysfs_id = "".to_owned();
            let item_kernel_driver = Self::get_kernel_driver(&item_sysfs_busid)?;
            let item_vendor_icon_name = "".to_owned();
    
            devices.push(Self {
                class_name: item_class,
                device_name: item_device,
                vendor_name: item_vendor,
                class_id:  item_class_id,
                device_id: item_device_id,
                vendor_id: item_vendor_id,
                sysfs_busid: item_sysfs_busid,
                sysfs_id: item_sysfs_id,
                kernel_driver: item_kernel_driver,
                vendor_icon_name: item_vendor_icon_name,
                available_profiles: Rc::default(),
            });
        }
    
        let mut uniq_devices = vec![];
        for device in devices.iter() {
            // Check if already in list
            let found = uniq_devices.iter().any(|x: &Self| {
                (device.sysfs_busid == x.sysfs_busid) && (device.sysfs_id == x.sysfs_id)
            });
    
            if !found {
                uniq_devices.push(device.clone());
            }
        }
        Some(uniq_devices)
    }

    pub fn create_class_hashmap(devices: Vec<Self>) -> HashMap<String, Vec<Self>> {
        let mut map: HashMap<String, Vec<Self>> = HashMap::new();

        for device in devices {
            // Use the entry API to get or create a Vec for the key
            map.entry(device.class_id.clone()).or_insert_with(Vec::new).push(device);
        }

        map
    }    
}

#[derive(Debug, Clone)]
pub struct CfhdbPciProfile {
    pub i18n_desc: String,
    pub icon_name: String,
    pub class_ids: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub device_ids: Vec<String>,
    pub blacklisted_class_ids: Vec<String>,
    pub blacklisted_vendor_ids: Vec<String>,
    pub blacklisted_device_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub priority: i32,
}
