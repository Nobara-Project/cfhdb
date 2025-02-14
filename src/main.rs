use libcfhdb::pci::*;
use libcfhdb::usb::*;

mod config;
mod pci_func;
mod usb_func;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

fn parse_args(args: Vec<String>) {
    let arg = args[1].as_str();
    match arg {
        "-h" | "--help" => {
            println!("help msg")
        }
        _ => {
            println!("unknown arg");
            std::process::exit(1);
        }
    }
}

fn main() {
    // Setup locales
    let current_locale = match std::env::var_os("LANG") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$LANG is not set"),
    };
    let args: Vec<String> = std::env::args().collect();
    let arg_num = args.len();
    match arg_num {
        0 | 1 => {
            println!("help msg")
        }
        2 => {
            parse_args(args);
        }
        _ => {
            println!("too much args");
            std::process::exit(1);
        }
    }
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
