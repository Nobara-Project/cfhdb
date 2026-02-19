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

// Implement Serialize for Arc<Mutex<Option<Vec<Arc<CfhdbPciProfile>>>>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Arc<Mutex<Option<Vec<Arc<CfhdbPciProfile>>>>>);
impl Serialize for ProfileWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Borrow the Mutex
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
    pub started: Option<bool>,
    pub enabled: bool,
    pub sysfs_busid: String,
    pub sysfs_id: String,
    pub kernel_driver: String,
    // Cfhdb Extras
    pub available_profiles: ProfileWrapper,
}

impl CfhdbPciDevice {
    fn get_kernel_driver(busid: &str) -> Option<String> {
        let device_uevent_path = format!("/sys/bus/pci/devices/{}/uevent", busid);
        match fs::read_to_string(device_uevent_path) {
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
        let mut available_profiles: Vec<Arc<CfhdbPciProfile>> = vec![];
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
                available_profiles.push(Arc::new(profile.clone()));
            };

            if !available_profiles.is_empty() {
                *device.available_profiles.0.lock().unwrap() = Some(available_profiles.clone());
            };
        }
    }

    fn get_started(busid: &str) -> Result<bool, io::Error> {
        let device_enable_path = std::path::Path::new("/sys/bus/pci/devices")
            .join(busid)
            .join("enable");
        let enable_status = fs::read_to_string(&device_enable_path)?;
        Ok(enable_status.trim() == "1")
    }

    fn get_enabled(busid: &str) -> bool {
        let pci_busid_blacklist_path = "/etc/cfhdb/pci_blacklist";
        match File::open(&pci_busid_blacklist_path) {
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
        let modalias = fs::read_to_string(format!("/sys/bus/pci/devices/{}/modalias", busid))?;
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
                "pci",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "stop_device",
                "pci",
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
                "pci",
                &self.sysfs_busid,
                Self::get_modinfo_name(&self.sysfs_busid).unwrap_or("".to_string())
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "start_device",
                "pci",
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
                "pci",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "enable_device",
                "pci",
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
                "pci",
                &self.sysfs_busid
            )
        } else {
            duct::cmd!(
                "pkexec",
                "/usr/lib/cfhdb/scripts/sysfs_helper.sh",
                "disable_device",
                "pci",
                &self.sysfs_busid
            )
        };
        cmd.run()?;
        Ok(())
    }

    pub fn get_device_from_busid(busid: &str) -> Result<CfhdbPciDevice, io::Error> {
        let devices = match CfhdbPciDevice::get_devices() {
            Some(t) => t,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Could not get pci devices",
                ));
            }
        };
        match devices.iter().find(|x| x.sysfs_busid == busid) {
            Some(device) => Ok(device.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no pci device with matching busid",
            )),
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
            let item_started = Self::get_started(&item_sysfs_busid);
            let item_enabled = Self::get_enabled(&item_sysfs_busid);
            let item_sysfs_id = "".to_owned();
            let item_kernel_driver =
                Self::get_kernel_driver(&item_sysfs_busid).unwrap_or("Unknown".to_string());

            devices.push(Self {
                class_name: item_class,
                device_name: item_device,
                vendor_name: item_vendor,
                class_id: item_class_id,
                device_id: item_device_id,
                vendor_id: item_vendor_id,
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
                sysfs_busid: item_sysfs_busid,
                sysfs_id: item_sysfs_id,
                kernel_driver: item_kernel_driver,
                available_profiles: ProfileWrapper(Arc::default()),
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
    pub veiled: bool,
    pub priority: i32,
}

impl CfhdbPciProfile {
    pub fn get_profile_from_codename(
        codename: &str,
        profiles: Vec<CfhdbPciProfile>,
    ) -> Result<Self, io::Error> {
        match profiles.iter().find(|x| x.codename == codename) {
            Some(profile) => Ok(profile.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no pci profile with matching codename",
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
