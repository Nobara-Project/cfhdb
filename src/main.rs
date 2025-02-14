use libcfhdb::pci::*;
use libcfhdb::usb::*;

mod config;
use config::PCI_PROFILE_JSON_URL;

fn get_pci_profiles_from_url() {
    let data = reqwest::blocking::get(PCI_PROFILE_JSON_URL)
        .unwrap()
        .text()
        .unwrap();
    let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    if let serde_json::Value::Array(profiles) = &res["profiles"] {
        for profile in profiles {
            let codename = &profile["codename"].as_str();
            /*let driver_name = driver["driver"].as_str().to_owned().unwrap().to_string();
            let driver_device = result.unwrap();
            let driver_icon = driver["icon"].as_str().to_owned().unwrap().to_string();
            let driver_experimental = driver["experimental"].as_bool().unwrap();
            let driver_removeble = driver["removable"].as_bool().unwrap();
            let command_version_label =
                Command::new("/usr/lib/pika/drivers/generate_package_info.sh")
                    .args(["version", &driver_name])
                    .output()
                    .unwrap();
            let command_description_label =
                Command::new("/usr/lib/pika/drivers/generate_package_info.sh")
                    .args(["description", &driver_name])
                    .output()
                    .unwrap();
            let found_driver_package = DriverPackage {
                driver: driver_name,
                version: String::from_utf8(command_version_label.stdout)
                    .unwrap()
                    .trim()
                    .to_string(),
                device: driver_device,
                description: String::from_utf8(command_description_label.stdout)
                    .unwrap()
                    .trim()
                    .to_string(),
                icon: driver_icon,
                experimental: driver_experimental,
                removeble: driver_removeble,
            };*/
            //driver_package_array.push(found_driver_package)

        }
    }
}

fn main_pci() {
    let devices = CfhdbPciDevice::get_devices().unwrap();
    for i in &devices {
        CfhdbPciDevice::set_available_profiles(&profiles, &i);
    }
    let hashmap = CfhdbPciDevice::create_class_hashmap(devices);
    dbg!(hashmap);
}

/*fn main() {
    let profiles = [
        CfhdbUsbProfile
        {
            i18n_desc: "Open source NVIDIA drivers for Linux (Latest)".to_owned(),
            icon_name: "".to_owned(),
            class_codes: ["0000".to_owned(),].to_vec(),
            vendor_ids: ["10de".to_owned()].to_vec(),
            product_ids: ["*".to_owned()].to_vec(),
            blacklisted_class_codes: [].to_vec(),
            blacklisted_vendor_ids: [].to_vec(),
            blacklisted_product_ids: [].to_vec(),
            packages: Some(["nvidie-driver-open-555".to_owned()].to_vec()),
            install_script: None,
            remove_script: None,
            priority: 10
        }
    ];
    let devices = CfhdbUsbDevice::get_devices().unwrap();
    for i in &devices {
        CfhdbUsbDevice::set_available_profiles(&profiles, &i);
    }
    let hashmap = CfhdbUsbDevice::create_class_hashmap(devices);
    dbg!(hashmap);
}*/
