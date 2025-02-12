use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct CfhdbPciDevice {
    pub class_name: String,
    pub device_name: String,
    pub vendor_name: String,
    pub class_id: String,
    pub vendor_id: String,
    pub device_id: String,
    pub sysfs_busid: String,
    pub sysfs_id: String,
    pub kernel_driver: String,
    pub vendor_icon_name: String,
    pub available_profiles: Rc<RefCell<Option<Vec<CfhdbDeviceProfile>>>>,
}

#[derive(Debug, Clone)]
struct CfhdbDeviceProfile {
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
    pub priority: i32,
}

fn get_pci_device_kernel_driver(busid: &str) -> String {
    let device_uevent_path = format!("/sys/bus/pci/devices/{}/uevent", busid);
    match std::fs::read_to_string(device_uevent_path) {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("DRIVER=") {
                    if let Some(value) = line.splitn(2, '=').nth(1) {
                        return value.to_string();
                    }
                }
            }
        }
        Err(_) => {
            return "???".to_owned();
        }
    }
    return "???".to_owned()
}

fn set_available_profiles(profile_data: &[CfhdbDeviceProfile], device: &CfhdbPciDevice) {
    let mut available_profiles: Vec<CfhdbDeviceProfile> = vec![];
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

fn generate_pci_devices() -> Option<Vec<CfhdbPciDevice>> {
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
        let item_kernel_driver = get_pci_device_kernel_driver(&item_sysfs_busid);
        let item_vendor_icon_name = "".to_owned();

        devices.push(CfhdbPciDevice {
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
        let found = uniq_devices.iter().any(|x: &CfhdbPciDevice| {
            (device.sysfs_busid == x.sysfs_busid) && (device.sysfs_id == x.sysfs_id)
        });

        if !found {
            uniq_devices.push(device.clone());
        }
    }
    Some(uniq_devices)
}

fn main() {
    let profiles = [
        CfhdbDeviceProfile
        {
            i18n_desc: "Open source NVIDIA drivers for Linux (Latest)".to_owned(),
            icon_name: "".to_owned(),
            class_ids: ["0300".to_owned(), "0302".to_owned(), "0380".to_owned()].to_vec(),
            vendor_ids: ["10de".to_owned()].to_vec(),
            device_ids: ["*".to_owned()].to_vec(),
            blacklisted_class_ids: ["0300".to_owned(), "0302".to_owned(), "0380".to_owned()].to_vec(),
            blacklisted_vendor_ids: ["".to_owned()].to_vec(),
            blacklisted_device_ids: ["".to_owned()].to_vec(),
            packages: Some(["nvidie-driver-open-555".to_owned()].to_vec()),
            install_script: None,
            remove_script: None,
            priority: 10
        },
        CfhdbDeviceProfile
        {
            i18n_desc: "Open source NVIDIA drivers for Linux (Latest)".to_owned(),
            icon_name: "".to_owned(),
            class_ids: ["0300".to_owned(), "0302".to_owned(), "0380".to_owned()].to_vec(),
            vendor_ids: ["10de".to_owned()].to_vec(),
            device_ids: ["*".to_owned()].to_vec(),
            blacklisted_class_ids: ["0300".to_owned(), "0302".to_owned(), "0380".to_owned()].to_vec(),
            blacklisted_vendor_ids: ["".to_owned()].to_vec(),
            blacklisted_device_ids: ["".to_owned()].to_vec(),
            packages: Some(["nvidie-driver-open-565".to_owned()].to_vec()),
            install_script: None,
            remove_script: None,
            priority: 10
        }
    ];
    let devices = generate_pci_devices().unwrap();
    for i in &devices {
        set_available_profiles(&profiles, &i);
    }
    dbg!(devices);
}