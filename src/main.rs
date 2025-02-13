use libcfhdb::pci::*;
use libcfhdb::usb::*;

fn main_pci() {
    let profiles = [
        CfhdbPciProfile
        {
            i18n_desc: "Open source NVIDIA drivers for Linux (Latest)".to_owned(),
            icon_name: "".to_owned(),
            class_ids: ["0300".to_owned(), "0302".to_owned(), "0380".to_owned()].to_vec(),
            vendor_ids: ["10de".to_owned()].to_vec(),
            device_ids: ["*".to_owned()].to_vec(),
            blacklisted_class_ids: [].to_vec(),
            blacklisted_vendor_ids: [].to_vec(),
            blacklisted_device_ids: [].to_vec(),
            packages: Some(["nvidie-driver-open-555".to_owned()].to_vec()),
            install_script: None,
            remove_script: None,
            priority: 10
        },
        CfhdbPciProfile
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
    let devices = CfhdbPciDevice::get_devices().unwrap();
    for i in &devices {
        CfhdbPciDevice::set_available_profiles(&profiles, &i);
    }
    let hashmap = CfhdbPciDevice::create_class_hashmap(devices);
    dbg!(hashmap);
}

fn main() {
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
}