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
            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let icon_name = profile["icon_name"].to_string();
            let class_ids: Vec<String> = profile["class_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let vendor_ids: Vec<String> = profile["vendor_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let device_ids: Vec<String> = profile["device_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let blacklisted_class_ids: Vec<String> = profile["blacklisted_class_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let blacklisted_vendor_ids: Vec<String> = profile["blacklisted_vendor_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let blacklisted_device_ids: Vec<String> = profile["blacklisted_device_ids"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect();
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(profile["packages"].as_array().expect("invalid_profile_class_ids").into_iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect())
            };
            let install_script_value = profile["install_script"].as_str().unwrap_or_default().to_string();
            let install_script = match install_script_value.as_str()  {
                "Option::is_none" => {
                    None
                }
                _ => {
                    Some(install_script_value)
                }
            };
            let remove_script_value = profile["remove_script"].as_str().unwrap_or_default().to_string();
            let remove_script = match remove_script_value.as_str()  {
                "Option::is_none" => {
                    None
                }
                _ => {
                    Some(remove_script_value)
                }
            };
            let experimental= profile["experimental"].as_bool().unwrap_or_default();
            let removable= profile["removable"].as_bool().unwrap_or_default();
            let priority= profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbPciProfile {
                codename,
                i18n_desc: "".to_string(),
                icon_name,
                class_ids,
                vendor_ids,
                device_ids,
                blacklisted_class_ids,
                blacklisted_vendor_ids,
                blacklisted_device_ids,
                packages,
                install_script,
                remove_script,
                experimental,
                removable,
                priority: priority as i32,
            };
            dbg!(profile_struct);
            //driver_package_array.push(found_driver_package)

        }
    }
}

fn main() {
    //let devices = CfhdbPciDevice::get_devices().unwrap();
    /*for i in &devices {
        CfhdbPciDevice::set_available_profiles(&profiles, &i);
    }*/
    //let hashmap = CfhdbPciDevice::create_class_hashmap(devices);
    //dbg!(hashmap);
    get_pci_profiles_from_url();
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
