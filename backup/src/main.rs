use std::rc::Rc;
use pci_info::PciInfo;

fn from_hex (hex_number:u32, fill: usize) -> String
{
    format!("{:01$x}", hex_number, fill)
}


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
    pub vendor_icon_name: String,
    pub os_driver: String,
    pub available_profiles: Vec<Rc<CfhdbDeviceProfile>>,
}

#[derive(Debug, Clone)]
struct CfhdbDeviceProfile {
    pub i18n_desc: String,
    pub icon_name: String,
    pub class_ids: Vec<String>,
    pub vendor_ids: Vec<String>,
    pub device_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub priority: i32,
}

fn get_pci_class_name(class: pci_info::pci_enums::PciDeviceClass) {

}

fn generate_pci_devices() -> Option<Vec<CfhdbPciDevice>> {
    // Get PCI devices
    let info = PciInfo::enumerate_pci().unwrap();
    let mut devices = vec![];
    let unknown_string = "???".to_string();

    for pci_entry in info {
        match pci_entry {
            Ok(iter) => {
                let item_class = iter.device_class().unwrap();
                let item_vendor = "".to_owned();//iter.vendor()?;
                let item_revision = match iter.revision() {
                    Ok(r) => " ".to_owned() + &r.to_string(),
                    Err(_) => "".to_owned()
                };
                let item_device = from_hex(iter.subsystem_vendor_id().unwrap().unwrap_or(1) as _, 4) + &item_revision;
                let item_class_id = from_hex(iter.device_class_code().unwrap() as _, 4);
                let item_device_id = from_hex(iter.device_id() as _, 4);
                let item_vendor_id = from_hex(iter.vendor_id() as _, 4);
                let item_sysfs_busid = iter.location().unwrap().to_string();
                let item_sysfs_id = "".to_owned();
                let item_vendor_icon_name = "".to_owned();
                let item_os_driver = iter.os_driver().unwrap().as_ref().unwrap_or(&unknown_string);
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
                    vendor_icon_name: item_vendor_icon_name,
                    os_driver: item_os_driver.clone(),
                    available_profiles: item_available_profiles,
                });
            }
            Err(err) => {
                eprintln!("{}", err)
            }    
        }
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
