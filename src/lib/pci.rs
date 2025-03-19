use serde::Serialize;
use serde::Serializer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Implement Serialize for Rc<RefCell<Option<Vec<Rc<CfhdbPciProfile>>>>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Rc<RefCell<Option<Vec<Rc<CfhdbPciProfile>>>>>);
impl Serialize for ProfileWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Borrow the RefCell
        let borrowed = self.0.borrow();

        // Handle the Option
        if let Some(profiles) = &*borrowed {
            let simplified: Vec<String> =
                profiles.iter().map(|rc| rc.codename.to_string()).collect();
            simplified.serialize(serializer)
        } else {
            // Serialize as null if the Option is None
            serializer.serialize_none()
        }
    }
}

#[derive(Serialize, Debug, Clone)]
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
    pub enabled: Option<bool>,
    pub sysfs_busid: String,
    pub sysfs_id: String,
    pub kernel_driver: String,
    // Cfhdb Extras
    pub vendor_icon_name: String,
    pub available_profiles: ProfileWrapper,
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
            Err(_) => {}
        }
        return None;
    }

    pub fn set_available_profiles(profile_data: &[CfhdbPciProfile], device: &Self) {
        let mut available_profiles: Vec<Rc<CfhdbPciProfile>> = vec![];
        for profile in profile_data.iter() {
            let matching = {
                if (profile.blacklisted_class_ids.contains(&"*".to_owned())
                    || profile.blacklisted_class_ids.contains(&device.class_id))
                    || (profile.blacklisted_vendor_ids.contains(&"*".to_owned())
                        || profile.blacklisted_vendor_ids.contains(&device.vendor_id))
                    || (profile.blacklisted_device_ids.contains(&"*".to_owned())
                        || profile.blacklisted_device_ids.contains(&device.device_id))
                {
                    false
                } else {
                    (profile.class_ids.contains(&"*".to_owned())
                        || profile.class_ids.contains(&device.class_id))
                        && (profile.vendor_ids.contains(&"*".to_owned())
                            || profile.vendor_ids.contains(&device.vendor_id))
                        && (profile.device_ids.contains(&"*".to_owned())
                            || profile.device_ids.contains(&device.device_id))
                }
            };

            if matching {
                available_profiles.push(Rc::new(profile.clone()));
            };

            if !available_profiles.is_empty() {
                *device.available_profiles.0.borrow_mut() = Some(available_profiles.clone());
            };
        }
    }

    fn get_enabled(busid: &str) -> Result<bool, std::io::Error> {
        let device_enable_path = std::path::Path::new("/sys/bus/pci/devices")
            .join(busid)
            .join("enable");
        let enable_status = std::fs::read_to_string(&device_enable_path)?;
        Ok(enable_status.trim() == "1")
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
            let item_class_id = from_hex(iter.class_id()? as _, 4).to_uppercase();
            let item_device_id = from_hex(iter.device_id()? as _, 4);
            let item_vendor_id = from_hex(iter.vendor_id()? as _, 4);
            let item_sysfs_busid = format!(
                "{}:{}:{}.{}",
                from_hex(iter.domain()? as _, 4),
                from_hex(iter.bus()? as _, 2),
                from_hex(iter.dev()? as _, 2),
                iter.func()?,
            );
            let item_enabled = Self::get_enabled(&item_sysfs_busid);
            let item_sysfs_id = "".to_owned();
            let item_kernel_driver =
                Self::get_kernel_driver(&item_sysfs_busid).unwrap_or("Unknown".to_string());
            let item_vendor_icon_name = "".to_owned();

            devices.push(Self {
                class_name: item_class,
                device_name: item_device,
                vendor_name: item_vendor,
                class_id: item_class_id,
                device_id: item_device_id,
                vendor_id: item_vendor_id,
                enabled: match item_enabled {
                    Ok(t) => {
                        if item_kernel_driver != "Unknown" {
                            Some(t)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                sysfs_busid: item_sysfs_busid,
                sysfs_id: item_sysfs_id,
                kernel_driver: item_kernel_driver,
                vendor_icon_name: item_vendor_icon_name,
                available_profiles: ProfileWrapper(Rc::default()),
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
            map.entry(device.class_id.clone())
                .or_insert_with(Vec::new)
                .push(device);
        }

        map
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CfhdbPciProfile {
    pub codename: String,
    pub i18n_desc: String,
    pub icon_name: String,
    pub license: String,
    pub class_ids: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub device_ids: Vec<String>,
    pub blacklisted_class_ids: Vec<String>,
    pub blacklisted_vendor_ids: Vec<String>,
    pub blacklisted_device_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub check_script: String,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub priority: i32,
}
