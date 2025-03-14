use libcfhdb::pci::*;
use libcfhdb::usb::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

mod config;
mod pci_func;
mod usb_func;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

fn parse_args(args: Vec<String>) {
    let mut arg1 = args[1].as_str();
    let mut arg2 = if args.len() > 2 {
        Some(args[2].as_str())
    } else {
        None
    };
    match arg2 {
        Some(t) => {
            if t == "-j" || t == "--json" {
                (arg1, arg2) = (t, Some(arg1));
            }
        }
        None => {}
    }
    match arg1 {
        "-h" | "--help" => {
            println!("{}", t!("help_msg"))
        }
        "-v" | "--version" => {
            println!("{}", VERSION)
        }
        "-j" | "--json" => {
            println!("help msg")
        }
        "-lpd" | "--list-pci-devices" => {
            match arg2 {
                Some(t) => {
                    match t {
                        "-j" | "--json" => {
                            println!("help msg")
                        }
                        _ => {
                            pci_func::display_pci_devices(false);
                        }
                    }
                }
                _ => {
                    pci_func::display_pci_devices(false);
                }
            }
        }
        "-lpp" | "--list-pci-profiles" => {
            match arg2 {
                Some(t) => {
                    match t {
                        "-j" | "--json" => {
                            println!("help msg")
                        }
                        _ => {
                            println!("help msg")
                        }
                    }
                }
                _ => {
                    
                }
            }
        }
        "-ipp" | "--install-pci-profile" => {
            println!("help msg")
        }
        "-upp" | "--uninstall-pci-profile" => {
            println!("help msg")
        }
        "-lup" | "--list-usb-profiles" => {
            match arg2 {
                Some(t) => {
                    match t {
                        "-j" | "--json" => {
                            println!("help msg")
                        }
                        _ => {
                            println!("help msg")
                        }
                    }
                }
                _ => {

                }
            }
        }
        "-lud" | "--list-usb-devices" => {
            match arg2 {
                Some(t) => {
                    match t {
                        "-j" | "--json" => {
                            println!("help msg")
                        }
                        _ => {
                            println!("help msg")
                        }
                    }
                }
                _ => {

                }
            }
        }
        "-iup" | "--install-usb-profile" => {
            println!("help msg")
        }
        "-uup" | "--uninstall-usb-profile" => {
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
    rust_i18n::set_locale(current_locale.strip_suffix(".UTF-8").unwrap());
    let args: Vec<String> = std::env::args().collect();
    let arg_num = args.len();
    match arg_num {
        0 | 1 => {
            println!("{}", t!("help_msg"))
        }
        _ => {
            parse_args(args);
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