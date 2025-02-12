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
    pub available_profiles: RefCell<Rc<Vec<CfhdbDeviceProfile>>>,
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

fn set_available_profiles(profile_data: &[&CfhdbDeviceProfile], device: &CfhdbPciDevice) {
    let mut available_profiles: Vec<&CfhdbDeviceProfile> = vec![];
    for profile in profile_data.iter() {
        let matching = available_profiles.iter().any(|x: &&CfhdbDeviceProfile| {
           (x.class_ids.contains(&"*".to_owned()) || x.class_ids.contains(&device.class_id))
           &&
           (x.vendor_ids.contains(&"*".to_owned()) || x.vendor_ids.contains(&device.vendor_id))
           &&
           (x.device_ids.contains(&"*".to_owned()) || x.device_ids.contains(&device.device_id))
           &&
           !(x.blacklisted_class_ids.contains(&"*".to_owned()) || x.blacklisted_class_ids.contains(&device.class_id))
           &&
           !(x.blacklisted_vendor_ids.contains(&"*".to_owned()) || x.blacklisted_vendor_ids.contains(&device.vendor_id))
           &&
           !(x.blacklisted_device_ids.contains(&"*".to_owned()) || x.blacklisted_device_ids.contains(&device.device_id))
        });

        if matching {
            available_profiles.push(profile.clone());
        }
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
        let item_class_id = from_hex(iter.dev()? as _, 4);
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
        let item_available_profiles = vec![];

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
            available_profiles: item_available_profiles,
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

/*
fn get_available_pci_profiles<'a>() -> [DeviceProfileEntry<'a>] {

}

fn main() {
    let nvidia_open_profile_555 = DeviceProfileEntry{
        i18n_desc: "Open source NVIDIA drivers for Linux (Latest)",
        class_ids: &[0300, 0302, 0380],
        vendor_ids: &["10de"],
        device_ids: &["*"],
        packages: Some(&["nvidie-driver-open-555"]),
        install_script: None,
        remove_script: None,
        priority: 10
    };
    dbg!(nvidia_open_profile_555);
}*/

/*pub fn main() {

    // Print out some properties of the enumerated devices.
    // Note that the collection contains both devices and errors
    // as the enumeration of PCI devices can fail entirely (in which
    // case `PciInfo::enumerate_pci()` would return error) or
    // partially (in which case an error would be inserted in the
    // result).
    /*for r in info {
        match r {
            Ok(device) => {
                print!(
                "===\nBrand {}\n Product {}\nRevision {}\nClass ID {}\nVendor ID\nDevice ID\n===\n",
                device.vendor_id(),
                device.subsystem_vendor_id().unwrap().clone().unwrap(),
                device.revision().unwrap(),
                //get_stringed_pci_fullclass(&device),
                ""
                )
            }
            Err(error) => eprintln!("{error}"),
        }
    }*/
}*/


fn main() {
    dbg!(generate_pci_devices().unwrap());
}
