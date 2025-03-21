use regex::Regex;
use rust_i18n::t;
use serde::Serialize;
use serde::Serializer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::exit;
use std::rc::Rc;
use users::get_current_username;

// Implement Serialize for Rc<RefCell<Option<Vec<Rc<CfhdbUsbProfile>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Rc<RefCell<Option<Vec<Rc<CfhdbUsbProfile>>>>>);
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
    pub started: Option<bool>,
    pub enabled: bool,
    pub speed: String,
    // Cfhdb Extras
    pub vendor_icon_name: String,
    pub available_profiles: ProfileWrapper,
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
                *device.available_profiles.0.borrow_mut() = Some(available_profiles.clone());
            };
        }
    }

    fn get_started(busid: &str) -> Result<bool, std::io::Error> {
        let device_enable_path = std::path::Path::new("/sys/bus/usb/devices")
            .join(busid)
            .join("enable");
        let enable_status = std::fs::read_to_string(&device_enable_path)?;
        Ok(enable_status.trim() == "1")
    }

    fn get_enabled(busid: &str) -> bool {
        let usb_busid_blacklist_path = "/etc/cfhdb/usb_blacklist";
        match File::open(&usb_busid_blacklist_path) {
            Ok(file) => {
                let reader = io::BufReader::new(file);
                for line in reader.lines() {
                    match line {
                        Ok(t) => {
                            if t.trim() == busid {
                                return false;
                            }
                        }
                        Err(_) => {}
                    };
                }
            }
            Err(_) => {}
        }
        return true;
    }

    fn get_modinfo_name(busid: &str) -> Result<String, std::io::Error> {
        let modalias = fs::read_to_string(format!("/sys/bus/usb/devices/{}/modalias", busid))?;
        let modinfo_cmd = duct::cmd!("modinfo", modalias);
        let stdout = modinfo_cmd.read()?;
        let re = Regex::new(r"name:\s+(\w+)").unwrap();
        for line in stdout.lines() {
            if let Some(captures) = re.captures(line) {
                // Extract the module name from the capture group
                if let Some(module_name) = captures.get(1) {
                    return Ok(module_name.as_str().to_string());
                }
            }
        }
        Err(std::io::Error::new(ErrorKind::NotFound, "not found"))
    }

    pub fn stop_device(&self) -> Result<(), std::io::Error> {
        let cmd = if get_current_username().unwrap() == "root" {
            duct::cmd!(
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "stop_device",
                "usb",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "stop_device",
                "usb",
                &self.sysfs_busid
            )
        };
        cmd.run()?;
        Ok(())
    }

    pub fn start_device(&self) -> Result<(), std::io::Error> {
        let cmd = if get_current_username().unwrap() == "root" {
            duct::cmd!(
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "start_device",
                "usb",
                &self.sysfs_busid,
                Self::get_modinfo_name(&self.sysfs_busid).unwrap_or("".to_string())
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "start_device",
                "usb",
                &self.sysfs_busid,
                Self::get_modinfo_name(&self.sysfs_busid).unwrap_or("".to_string())
            )
        };
        cmd.run()?;
        Ok(())
    }

    pub fn enable_device(&self) -> Result<(), std::io::Error> {
        let cmd = if get_current_username().unwrap() == "root" {
            duct::cmd!(
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "enable_device",
                "usb",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "enable_device",
                "usb",
                &self.sysfs_busid
            )
        };
        cmd.run()?;
        Ok(())
    }

    pub fn disable_device(&self) -> Result<(), std::io::Error> {
        let cmd = if get_current_username().unwrap() == "root" {
            duct::cmd!(
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "disable_device",
                "usb",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "disable_device",
                "usb",
                &self.sysfs_busid
            )
        };
        cmd.run()?;
        Ok(())
    }

    pub fn get_device_from_busid(busid: &str) -> Result<CfhdbUsbDevice, std::io::Error> {
        let devices = match CfhdbUsbDevice::get_devices() {
            Some(t) => t,
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Could not get usb devices",
                ));
            }
        };
        match devices.iter().find(|x| x.sysfs_busid == busid) {
            Some(device) => Ok(device.clone()),
            None => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "no usb device with matching busid",
            )),
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
            let item_started = Self::get_started(&item_sysfs_busid);
            let item_enabled = Self::get_enabled(&item_sysfs_busid);
            let item_serial_number_string_index = "".to_owned();
            let item_protocol_code = from_hex(device_descriptor.protocol_code() as _, 4);
            let item_class_code = from_hex(device_descriptor.class_code() as _, 4).to_uppercase();
            let item_vendor_id = from_hex(device_descriptor.vendor_id() as _, 4);
            let item_product_id = from_hex(device_descriptor.product_id() as _, 4);
            let item_usb_version = device_descriptor.usb_version().to_string();
            let item_port_number = iter.port_number();
            let item_kernel_driver =
                Self::get_kernel_driver(&item_sysfs_busid).unwrap_or("Unknown".to_string());
            let item_speed = match iter.speed() {
                rusb::Speed::Low => "1.0",
                rusb::Speed::Full=> "1.1",
                rusb::Speed::High => "2.0",
                rusb::Speed::Super=> "3.0",
                rusb::Speed::SuperPlus=> "3.1",
                _ => "Unknown"
            };
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
                kernel_driver: item_kernel_driver.clone(),
                started: match item_started {
                    Ok(t) => {
                        if item_kernel_driver != "Unknown" {
                            Some(t)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                enabled: item_enabled,
                speed: item_speed.to_string(),
                vendor_icon_name: item_vendor_icon_name,
                available_profiles: ProfileWrapper(Rc::default()),
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
    pub license: String,
    pub class_codes: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub product_ids: Vec<String>,
    pub blacklisted_class_codes: Vec<String>,
    pub blacklisted_vendor_ids: Vec<String>,
    pub blacklisted_product_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub check_script: String,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub priority: i32,
}

impl CfhdbUsbProfile {
    pub fn get_profile_from_codename(
        codename: &str,
        profiles: Vec<CfhdbUsbProfile>,
    ) -> Result<Self, std::io::Error> {
        match profiles.iter().find(|x| x.codename == codename) {
            Some(profile) => Ok(profile.clone()),
            None => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "no usb profile with matching codename",
            )),
        }
    }

    pub fn get_status(&self) -> bool {
        let file_path = "/var/cache/cfhdb/check_cmd.sh";
        {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)
                .expect(&(file_path.to_string() + "cannot be read"));
            file.write_all(
                format!(
                    "#! /bin/bash\nset -e\n{} > /dev/null 2>&1",
                    self.check_script
                )
                    .as_bytes(),
            )
                .expect(&(file_path.to_string() + "cannot be written to"));
            let mut perms = file
                .metadata()
                .expect(&(file_path.to_string() + "cannot be read"))
                .permissions();
            perms.set_mode(0o777);
            fs::set_permissions(file_path, perms)
                .expect(&(file_path.to_string() + "cannot be written to"));
        }
        duct::cmd!("bash", "-c", file_path).run().is_ok()
    }
}

