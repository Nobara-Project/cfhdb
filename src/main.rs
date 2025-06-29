use std::{fs, io::Write, os::unix::fs::PermissionsExt, path::Path, process::exit};

use cli_table::{format::Justify, Cell, Color, Style, Table};
use colored::Colorize;
use serde::Deserialize;
use users::get_current_username;

const VERSION: &str = env!("CARGO_PKG_VERSION");

mod config;
mod pci_func;
mod usb_func;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

#[derive(Deserialize)]
pub struct ProfileUrlConfig {
    pci_json_url: String,
    usb_json_url: String,
}

fn print_help_msg() {
    let table = vec![
        // Secondary titles
        vec![
            t!("help_msg_title1")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Blue)),
            t!("help_msg_title2")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Blue)),
            t!("help_msg_title3")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Blue)),
        ],
        // Program arguments title
        vec![
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("help_msg_title_program")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
        ],
        // Program arguments entries
        vec![
            t!("help_msg_action_help").cell(),
            "--help".cell(),
            "-h".cell(),
        ],
        vec![
            t!("help_msg_action_version").cell(),
            "--version".cell(),
            "-v".cell(),
        ],
        vec![
            t!("help_msg_action_json").cell(),
            "--json".cell(),
            "-j".cell(),
        ],
        // PCI arguments title
        vec![
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("help_msg_title_pci")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
        ],
        // PCI arguments entries
        vec![
            t!("help_msg_action_list_pci_devices").cell(),
            "--list-pci-devices".cell(),
            "-lpd".cell(),
        ],
        vec![
            t!("help_msg_action_list_compatible_pci_profiles").cell(),
            "--list-pci-profiles {sysfs_id}".cell(),
            "-lpp".cell(),
        ],
        vec![
            t!("help_msg_action_install_pci_profile").cell(),
            "--install-pci-profile {profile codename}".cell(),
            "-ipp".cell(),
        ],
        vec![
            t!("help_msg_action_uninstall_pci_profile").cell(),
            "--uninstall-pci-profile {profile codename}".cell(),
            "-upp".cell(),
        ],
        vec![
            t!("help_msg_action_enable_pci_device").cell(),
            "--enable-pci-device {sysfs_id}".cell(),
            "-epd".cell(),
        ],
        vec![
            t!("help_msg_action_disable_pci_device").cell(),
            "--disable-pci-device {sysfs_id}".cell(),
            "-dpd".cell(),
        ],
        vec![
            t!("help_msg_action_start_pci_device").cell(),
            "--start-pci-device {sysfs_id}".cell(),
            "-sspd".cell(),
        ],
        vec![
            t!("help_msg_action_stop_pci_device").cell(),
            "--stop-pci-device {sysfs_id}".cell(),
            "-srpd".cell(),
        ],
        // USB arguments title
        vec![
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("help_msg_title_usb")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
            t!("")
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .foreground_color(Some(Color::Yellow)),
        ],
        // USB arguments entries
        vec![
            t!("help_msg_action_list_usb_devices").cell(),
            "--list-usb-devices".cell(),
            "-lud".cell(),
        ],
        vec![
            t!("help_msg_action_list_compatible_usb_profiles").cell(),
            "--list-usb-profiles {sysfs_id}".cell(),
            "-lup".cell(),
        ],
        vec![
            t!("help_msg_action_install_usb_profile").cell(),
            "--install-usb-profile {profile codename}".cell(),
            "-iup".cell(),
        ],
        vec![
            t!("help_msg_action_uninstall_usb_profile").cell(),
            "--uninstall-usb-profile {profile codename}".cell(),
            "-uup".cell(),
        ],
        vec![
            t!("help_msg_action_enable_usb_device").cell(),
            "--enable-usb-device {sysfs_id}".cell(),
            "-eud".cell(),
        ],
        vec![
            t!("help_msg_action_disable_usb_device").cell(),
            "--disable-usb-device {sysfs_id}".cell(),
            "-dud".cell(),
        ],
        vec![
            t!("help_msg_action_start_usb_device").cell(),
            "--start-usb-device {sysfs_id}".cell(),
            "-ssud".cell(),
        ],
        vec![
            t!("help_msg_action_stop_usb_device").cell(),
            "--stop-usb-device {sysfs_id}".cell(),
            "-srud".cell(),
        ],
    ]
    .table()
    .title(vec![
        t!("help_msg_title0")
            .cell()
            .bold(true)
            .justify(Justify::Center),
        VERSION.cell().bold(true).justify(Justify::Center),
        "".cell().bold(true).justify(Justify::Center),
    ])
    .bold(true);

    let table_display = table.display().unwrap();

    println!("{}", table_display);
}
fn parse_args(args: Vec<String>) {
    let mut json_mode = false;
    let mut action = "-h";
    let mut additional_arguments = vec![];
    for arg in args {
        match arg.as_str() {
            // Global modes
            "-j" | "--json" => json_mode = true,
            // Program arguments
            "-h" | "--help" => action = "h",
            "-v" | "--version" => action = "v",
            // PCI arguments
            "-lpd" | "--list-pci-devices" => action = "lpd",
            "-lpp" | "--list-pci-profiles" => action = "lpp",
            "-ipp" | "--install-pci-profile" => action = "ipp",
            "-upp" | "--uninstall-pci-profile" => action = "upp",
            "-epd" | "--enable-pci-device" => action = "epd",
            "-dpd" | "--disable-pci-device" => action = "dpd",
            "-sspd" | "--start-pci-device" => action = "sspd",
            "-srpd" | "--stop-pci-device" => action = "srpd",
            // USB arguments
            "-lud" | "--list-usb-devices" => action = "lud",
            "-lup" | "--list-usb-profiles" => action = "lup",
            "-iup" | "--install-usb-profile" => action = "iup",
            "-uup" | "--uninstall-usb-profile" => action = "uup",
            "-eud" | "--enable-usb-device" => action = "eud",
            "-dud" | "--disable-usb-device" => action = "dud",
            "-ssud" | "--start-usb-device" => action = "ssud",
            "-srud" | "--stop-usb-device" => action = "srud",
            _ => {
                additional_arguments.push(arg);
            }
        }
    }
    match action {
        // Program arguments
        "h" => print_help_msg(),
        "v" => {
            println!("{}", VERSION)
        }
        "j" => print_help_msg(),
        // PCI arguments
        "lpd" => {
            pci_func::display_pci_devices(json_mode);
        }
        "lpp" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                pci_func::display_pci_profiles(json_mode, &additional_arguments[1]);
            }
        }
        "ipp" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_profile_specified"));
                std::process::exit(1);
            } else {
                pci_func::install_pci_profile(&additional_arguments[1]);
            }
        }
        "upp" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_profile_specified"));
                std::process::exit(1);
            } else {
                pci_func::uninstall_pci_profile(&additional_arguments[1]);
            }
        }
        "epd" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                pci_func::enable_pci_device(&additional_arguments[1]);
            }
        }
        "dpd" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                pci_func::disable_pci_device(&additional_arguments[1]);
            }
        }
        "sspd" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                pci_func::start_pci_device(&additional_arguments[1]);
            }
        }
        "srpd" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                pci_func::stop_pci_device(&additional_arguments[1]);
            }
        }
        // USB arguments
        "lud" => {
            usb_func::display_usb_devices(json_mode);
        }
        "lup" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                usb_func::display_usb_profiles(json_mode, &additional_arguments[1]);
            }
        }
        "iup" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_profile_specified"));
                std::process::exit(1);
            } else {
                usb_func::install_usb_profile(&additional_arguments[1]);
            }
        }
        "uup" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_profile_specified"));
                std::process::exit(1);
            } else {
                usb_func::uninstall_usb_profile(&additional_arguments[1]);
            }
        }
        "eud" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                usb_func::enable_usb_device(&additional_arguments[1]);
            }
        }
        "dud" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                usb_func::disable_usb_device(&additional_arguments[1]);
            }
        }
        "ssud" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                usb_func::start_usb_device(&additional_arguments[1]);
            }
        }
        "srud" => {
            if additional_arguments.len() < 2 {
                eprintln!("{}", t!("no_device_specified"));
                std::process::exit(1);
            } else {
                usb_func::stop_usb_device(&additional_arguments[1]);
            }
        }
        // Unknown argument
        _ => {
            eprintln!("{}", t!("unknown_argument"));
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
            print_help_msg();
        }
        _ => {
            parse_args(args);
        }
    }
}

pub fn run_in_lock_script(script: &str) {
    let file_path = "/var/cache/cfhdb/script_lock.sh";
    let file_fs_path = Path::new(file_path);
    if file_fs_path.exists() {
        fs::remove_file(file_fs_path).unwrap();
    }
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect(&(file_path.to_string() + "cannot be read"));
        file.write_all(script.as_bytes())
            .expect(&(file_path.to_string() + "cannot be written to"));
        let mut perms = file
            .metadata()
            .expect(&(file_path.to_string() + "cannot be read"))
            .permissions();
        perms.set_mode(0o777);
        fs::set_permissions(file_path, perms)
            .expect(&(file_path.to_string() + "cannot be written to"));
    }
    let final_cmd = if get_current_username().unwrap() == "root" {
        duct::cmd!(file_path)
    } else {
        duct::cmd!("pkexec", file_path)
    };
    match final_cmd.run() {
        Ok(_) => {
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("install_script_successful")
            );
            fs::remove_file(file_fs_path).unwrap();
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("install_script_failed"));
            fs::remove_file(file_fs_path).unwrap();
            exit(1);
        }
    }
}

pub fn get_profile_url_config() -> ProfileUrlConfig {
    let file_path = "/etc/cfhdb/profile-config.json";
    let json_content = fs::read_to_string(file_path).unwrap();
    let config: ProfileUrlConfig = serde_json::from_str(&json_content).unwrap();
    config
}
