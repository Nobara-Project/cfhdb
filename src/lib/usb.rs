use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CfhdbUsbDevice {
    // String identification
    pub class_name: String,
    pub manufacturer_string_index: String,
    pub product_string_index: String,
    pub serial_number_string_index: String,
    // Vendor IDs
    pub protocol_code: String,
    pub class_code: String,
    pub vendor_id: String,
    pub product_id: String,
    // System Info
    pub usb_version: String,
    pub bus_number: u8,
    pub port_number: u8,
    pub address: u8,
    pub sysfs_busid: String,
    pub kernel_driver: String,
    pub enabled: bool,
    pub speed: rusb::Speed,
    // Cfhdb Extras
    pub vendor_icon_name: String,
    pub available_profiles: Rc<RefCell<Option<Vec<Rc<CfhdbUsbProfile>>>>>,
}
impl CfhdbUsbDevice {
    fn get_sysfs_id(bus_number: u8, device_address: u8) -> Option<String> {
        // Base sysfs path
        let base_path = "/sys/bus/usb/devices";

        // Iterate over all entries in the base path
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    // Check if the entry starts with the bus number
                    if file_name.starts_with(&format!("{}-", bus_number)) {
                        // Read the "devnum" file to get the device address
                        let devnum_path = path.join("devnum");
                        if let Ok(devnum) = std::fs::read_to_string(devnum_path) {
                            let devnum = devnum.trim().parse::<u8>().unwrap_or(0);
                            if devnum == device_address {
                                // Return just the ID (e.g., "3-1.2")
                                return Some(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn get_kernel_driver(busid: &str) -> Option<String> {
        let device_driver_path = std::path::Path::new("/sys/bus/usb/devices")
            .join(busid)
            .join("driver");
        if device_driver_path.exists() {
            std::fs::read_link(device_driver_path)
                .ok()
                .and_then(|link| link.file_name().map(|s| s.to_string_lossy().into_owned()))
        } else {
            None
        }
    }

    fn get_manufacturer(busid: &str) -> Result<String, std::io::Error> {
        let device_manufacturer_path = std::path::Path::new("/sys/bus/usb/devices")
            .join(busid)
            .join("manufacturer");
        if device_manufacturer_path.exists() {
            std::fs::read_to_string(device_manufacturer_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Manufacturer file could not be found!",
            ))
        }
    }

    fn get_product(busid: &str) -> Result<String, std::io::Error> {
        let device_product_path = std::path::Path::new("/sys/bus/usb/devices")
            .join(busid)
            .join("product");
        if device_product_path.exists() {
            std::fs::read_to_string(device_product_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Product file could not be found!",
            ))
        }
    }

    pub fn set_available_profiles(profile_data: &[CfhdbUsbProfile], device: &Self) {
        let mut available_profiles: Vec<Rc<CfhdbUsbProfile>> = vec![];
        for profile in profile_data.iter() {
            let matching = {
                if (profile.blacklisted_class_codes.contains(&"*".to_owned())
                    || profile.blacklisted_class_codes.contains(&device.class_code))
                    || (profile.blacklisted_vendor_ids.contains(&"*".to_owned())
                        || profile.blacklisted_vendor_ids.contains(&device.vendor_id))
                    || (profile.blacklisted_product_ids.contains(&"*".to_owned())
                        || profile.blacklisted_product_ids.contains(&device.product_id))
                {
                    false
                } else {
                    (profile.class_codes.contains(&"*".to_owned())
                        || profile.class_codes.contains(&device.class_code))
                        && (profile.vendor_ids.contains(&"*".to_owned())
                            || profile.vendor_ids.contains(&device.vendor_id))
                        && (profile.product_ids.contains(&"*".to_owned())
                            || profile.product_ids.contains(&device.product_id))
                }
            };

            if matching {
                available_profiles.push(Rc::new(profile.clone()));
            };

            if !available_profiles.is_empty() {
                *device.available_profiles.borrow_mut() = Some(available_profiles.clone());
            };
        }
    }

    pub fn get_devices() -> Option<Vec<Self>> {
        let from_hex =
            |hex_number: u32, fill: usize| -> String { format!("{:01$x}", hex_number, fill) };

        // Get hardware devices
        let usb_devices = rusb::devices().unwrap();
        let mut devices = vec![];

        for iter in usb_devices.iter() {
            let device_descriptor = iter.device_descriptor().unwrap();

            let item_bus_number = iter.bus_number();
            let item_address = iter.address();
            let item_sysfs_busid =
                Self::get_sysfs_id(item_bus_number, item_address).unwrap_or("???".to_owned()); //format!("{}-{}-{}", iter.bus_number(), iter.port_number(), iter.address());

            let item_class_name = "".to_owned();
            let item_manufacturer_string_index = match Self::get_manufacturer(&item_sysfs_busid) {
                Ok(t) => t.trim().to_string(),
                Err(_) => "???".to_owned(),
            };
            let item_product_string_index = match Self::get_product(&item_sysfs_busid) {
                Ok(t) => t.trim().to_string(),
                Err(_) => "???".to_owned(),
            };
            let item_serial_number_string_index = "".to_owned();
            let item_protocol_code = from_hex(device_descriptor.protocol_code() as _, 4);
            let item_class_code = from_hex(device_descriptor.class_code() as _, 4).to_uppercase();
            let item_vendor_id = from_hex(device_descriptor.vendor_id() as _, 4);
            let item_product_id = from_hex(device_descriptor.product_id() as _, 4);
            let item_usb_version = device_descriptor.usb_version().to_string();
            let item_port_number = iter.port_number();
            let item_kernel_driver = match Self::get_kernel_driver(&item_sysfs_busid) {
                Some(t) => t,
                None => "???".to_owned(),
            };
            let item_speed = iter.speed();
            let item_vendor_icon_name = "".to_owned();

            devices.push(Self {
                class_name: item_class_name,
                manufacturer_string_index: item_manufacturer_string_index,
                product_string_index: item_product_string_index,
                serial_number_string_index: item_serial_number_string_index,
                protocol_code: item_protocol_code,
                class_code: item_class_code,
                vendor_id: item_vendor_id,
                product_id: item_product_id,
                usb_version: item_usb_version,
                sysfs_busid: item_sysfs_busid,
                bus_number: item_bus_number,
                port_number: item_port_number,
                address: item_address,
                kernel_driver: item_kernel_driver,
                enabled: true,
                speed: item_speed,
                vendor_icon_name: item_vendor_icon_name,
                available_profiles: Rc::default(),
            });
        }

        let mut uniq_devices = vec![];
        for device in devices.iter() {
            //Check if already in list
            let found = uniq_devices
                .iter()
                .any(|x: &Self| device.sysfs_busid == x.sysfs_busid);

            if !found && device.sysfs_busid != "???" {
                uniq_devices.push(device.clone());
            }
        }
        Some(uniq_devices)
    }
    pub fn create_class_hashmap(devices: Vec<Self>) -> HashMap<String, Vec<Self>> {
        let mut map: HashMap<String, Vec<Self>> = HashMap::new();

        for device in devices {
            // Use the entry API to get or create a Vec for the key
            map.entry(device.class_code.clone())
                .or_insert_with(Vec::new)
                .push(device);
        }

        map
    }
}

#[derive(Debug, Clone)]
pub struct CfhdbUsbProfile {
    pub codename: String,
    pub i18n_desc: String,
    pub icon_name: String,
    pub class_codes: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub product_ids: Vec<String>,
    pub blacklisted_class_codes: Vec<String>,
    pub blacklisted_vendor_ids: Vec<String>,
    pub blacklisted_product_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub priority: i32,
}
