use regex::Regex;
use serde::{Serialize, Serializer};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, ErrorKind, Write},
    os::unix::fs::PermissionsExt,
    sync::{Arc, Mutex},
};
use users::get_current_username;

// Implement Serialize for Rc<RefCell<Option<Vec<Rc<CfhdbUsbProfile>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Arc<Mutex<Option<Vec<Arc<CfhdbUsbProfile>>>>>);
impl Serialize for ProfileWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Borrow the RefCell
        let borrowed = self.0.lock().unwrap();

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

fn from_hex(hex_number: u32, fill: usize) -> String {
    format!("{:01$x}", hex_number, fill)
}

fn parse_from_lsusb_output() -> Vec<LsUsbEntry> {
    let output = std::process::Command::new("lsusb")
        .arg("-v")
        .output()
        .expect("Failed to execute lsusb");
    let output = std::str::from_utf8(&output.stdout).expect("Invalid UTF-8 in lsusb output");

    let mut did_first_header = false;
    let mut lsusb_entries = vec![];

    let mut current_vendor_id = None;
    let mut current_product_id = None;
    let mut current_vendor_name = None;
    let mut current_product_name = None;
    let mut current_interface_class = None;

    for line in output.lines() {
        let line = line.trim();
        if !did_first_header {
            if line.starts_with("Device Descriptor:") {
                did_first_header = true
            }
        } else if !line.starts_with("Device Descriptor:") {
            if line.starts_with("idVendor") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                current_vendor_id = parts[1].strip_prefix("0x");
                current_vendor_name = Some(parts[2..].join(" "));
            }
            if line.starts_with("idProduct") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                current_product_id = parts[1].strip_prefix("0x");
                current_product_name = Some(parts[2..].join(" "));
            }
            if line.starts_with("bInterfaceClass") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                current_interface_class = Some(
                    from_hex(parts[1].to_string().parse::<u32>().unwrap_or(00) as _, 2)
                        .to_uppercase(),
                );
            }
        } else {
            match (current_vendor_id, current_product_id) {
                (Some(a), Some(b)) => {
                    let entry = LsUsbEntry {
                        vendor_id: a.to_string(),
                        product_id: b.to_string(),
                        vendor_name: current_vendor_name,
                        product_name: current_product_name,
                        interface_class: current_interface_class.unwrap_or("00".to_string()),
                    };
                    lsusb_entries.push(entry);
                }
                (_, _) => {}
            }
            current_vendor_id = None;
            current_product_id = None;
            current_vendor_name = None;
            current_product_name = None;
            current_interface_class = None;
        }
    }
    match (current_vendor_id, current_product_id) {
        (Some(a), Some(b)) => {
            let entry = LsUsbEntry {
                vendor_id: a.to_string(),
                product_id: b.to_string(),
                vendor_name: current_vendor_name,
                product_name: current_product_name,
                interface_class: current_interface_class.unwrap_or("00".to_string()),
            };
            lsusb_entries.push(entry);
        }
        (_, _) => {}
    }

    lsusb_entries
}

#[derive(Debug, Clone)]
struct LsUsbEntry {
    vendor_id: String,
    product_id: String,
    vendor_name: Option<String>,
    product_name: Option<String>,
    interface_class: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CfhdbUsbDevice {
    // String identification
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
    pub available_profiles: ProfileWrapper,
}
impl CfhdbUsbDevice {
    fn get_sysfs_id(bus_number: u8, device_address: u8) -> Option<String> {
        // Base sysfs path
        let base_path = "/sys/bus/usb/devices";

        // Iterate over all entries in the base path
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    // Check if the entry starts with the bus number
                    if file_name.starts_with(&format!("{}-", bus_number)) {
                        // Read the "devnum" file to get the device address
                        let devnum_path = path.join("devnum");
                        if let Ok(devnum) = fs::read_to_string(devnum_path) {
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
        let device_driver_format = format!("/sys/bus/usb/devices/{}:1.0/driver", busid);
        let device_driver_path = std::path::Path::new(&device_driver_format);
        if device_driver_path.exists() {
            fs::read_link(device_driver_path)
                .ok()
                .and_then(|link| link.file_name().map(|s| s.to_string_lossy().into_owned()))
        } else {
            None
        }
    }

    fn get_serial(busid: &str) -> Result<String, io::Error> {
        let device_manufacturer_path = std::path::Path::new("/sys/bus/usb/devices")
            .join(busid)
            .join("serial");
        if device_manufacturer_path.exists() {
            match fs::read_to_string(device_manufacturer_path) {
                Ok(t) => Ok(t.trim().to_string()),
                Err(e) => Err(io::Error::new(ErrorKind::NotFound, e)),
            }
        } else {
            Err(io::Error::new(
                ErrorKind::NotFound,
                "serial file could not be found!",
            ))
        }
    }

    pub fn set_available_profiles(profile_data: &[CfhdbUsbProfile], device: &Self) {
        let mut available_profiles: Vec<Arc<CfhdbUsbProfile>> = vec![];
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
                available_profiles.push(Arc::new(profile.clone()));
            };

            if !available_profiles.is_empty() {
                *device.available_profiles.0.lock().unwrap() = Some(available_profiles.clone());
            };
        }
    }

    fn get_started(busid: &str) -> bool {
        let device_driver_format = format!("/sys/bus/usb/devices/{}:1.0/driver", busid);
        let device_driver_path = std::path::Path::new(&device_driver_format);
        device_driver_path.exists()
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

    fn get_modinfo_name(busid: &str) -> Result<String, io::Error> {
        let modalias = fs::read_to_string(format!("/sys/bus/usb/devices/{}:1.0/modalias", busid))?;
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
        Err(io::Error::new(ErrorKind::NotFound, "not found"))
    }

    pub fn stop_device(&self) -> Result<(), io::Error> {
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

    pub fn start_device(&self) -> Result<(), io::Error> {
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

    pub fn enable_device(&self) -> Result<(), io::Error> {
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

    pub fn disable_device(&self) -> Result<(), io::Error> {
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

    pub fn get_device_from_busid(busid: &str) -> Result<CfhdbUsbDevice, io::Error> {
        let devices = match CfhdbUsbDevice::get_devices() {
            Some(t) => t,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Could not get usb devices",
                ));
            }
        };
        match devices.iter().find(|x| x.sysfs_busid == busid) {
            Some(device) => Ok(device.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no usb device with matching busid",
            )),
        }
    }

    pub fn get_devices() -> Option<Vec<Self>> {
        let lsusb_entries = parse_from_lsusb_output();
        // Get hardware devices
        let usb_devices = rusb::devices().unwrap();
        let mut devices = vec![];

        for iter in usb_devices.iter() {
            let device_descriptor = iter.device_descriptor().unwrap();

            let item_bus_number = iter.bus_number();
            let item_address = iter.address();
            let item_sysfs_busid =
                Self::get_sysfs_id(item_bus_number, item_address).unwrap_or("???".to_owned()); //format!("{}-{}-{}", iter.bus_number(), iter.port_number(), iter.address());
            let item_vendor_id = from_hex(device_descriptor.vendor_id() as _, 4);
            let item_product_id = from_hex(device_descriptor.product_id() as _, 4);
            let item_lsusb_entry = lsusb_entries
                .iter()
                .cloned()
                .find(|x| x.vendor_id == item_vendor_id && x.product_id == item_product_id);
            let (item_manufacturer_string_index, item_product_string_index, item_class_code) =
                match item_lsusb_entry {
                    Some(t) => match (t.vendor_name, t.product_name) {
                        (Some(a), Some(b)) => (a, b, t.interface_class),
                        (_, _) => ("???".to_owned(), "???".to_owned(), t.interface_class),
                    },
                    None => ("???".to_owned(), "???".to_owned(), "00".to_owned()),
                };
            let item_started = Self::get_started(&item_sysfs_busid);
            let item_enabled = Self::get_enabled(&item_sysfs_busid);
            let item_serial_number_string_index =
                Self::get_serial(&item_sysfs_busid).unwrap_or("Unknown".to_string());
            let item_protocol_code = from_hex(device_descriptor.protocol_code() as _, 4);
            //let item_class_code = (from_hex(device_descriptor.class_code() as _, 2) + &from_hex(device_descriptor.sub_class_code() as _, 2)).to_uppercase();
            //let item_class_code = from_hex(device_descriptor.class_code() as _, 2).to_uppercase();
            let item_usb_version = device_descriptor.usb_version().to_string();
            let item_port_number = iter.port_number();
            let item_kernel_driver =
                Self::get_kernel_driver(&item_sysfs_busid).unwrap_or("Unknown".to_string());
            let item_speed = match iter.speed() {
                rusb::Speed::Low => "1.0",
                rusb::Speed::Full => "1.1",
                rusb::Speed::High => "2.0",
                rusb::Speed::Super => "3.0",
                rusb::Speed::SuperPlus => "3.1",
                _ => "Unknown",
            };
            devices.push(Self {
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
                started: if item_kernel_driver != "Unknown" {
                    Some(item_started)
                } else {
                    None
                },
                enabled: item_enabled,
                speed: item_speed.to_string(),
                available_profiles: ProfileWrapper(Arc::default()),
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
    pub veiled: bool,
    pub priority: i32,
}

impl CfhdbUsbProfile {
    pub fn get_profile_from_codename(
        codename: &str,
        profiles: Vec<CfhdbUsbProfile>,
    ) -> Result<Self, io::Error> {
        match profiles.iter().find(|x| x.codename == codename) {
            Some(profile) => Ok(profile.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no usb profile with matching codename",
            )),
        }
    }

    pub fn get_status(&self) -> bool {
        let file_path = "/var/cache/cfhdb/check_cmd.sh";
        {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)
                .expect(&(file_path.to_string() + "cannot be read"));
            file.write_all(format!("#! /bin/bash\nset -e\n{}", self.check_script).as_bytes())
                .expect(&(file_path.to_string() + "cannot be written to"));
            let mut perms = file
                .metadata()
                .expect(&(file_path.to_string() + "cannot be read"))
                .permissions();
            perms.set_mode(0o777);
            fs::set_permissions(file_path, perms)
                .expect(&(file_path.to_string() + "cannot be written to"));
        }
        duct::cmd!("bash", "-c", file_path)
            .stderr_to_stdout()
            .stdout_null()
            .run()
            .is_ok()
    }
}
