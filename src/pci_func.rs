use crate::config::*;
use cli_table::{format::Justify, Cell, Color, Style, Table};
use colored::Colorize;
use libcfhdb::pci::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::process::exit;

fn display_pci_devices_print_json(hashmap: HashMap<String, Vec<CfhdbPciDevice>>) {
    let json_pretty = serde_json::to_string_pretty(&hashmap).unwrap();
    println!("{}", json_pretty);
}
fn display_pci_devices_print_cli_table(hashmap: HashMap<String, Vec<CfhdbPciDevice>>) {
    for (class, devices) in hashmap {
        let mut table_struct = vec![];
        for device in devices {
            let cell_table = vec![
                match device.vendor_name.char_indices().nth(18) {
                    None => device.vendor_name,
                    Some((idx, _)) => device.vendor_name[..idx].to_string() + "...",
                }
                .cell(),
                match device.device_name.char_indices().nth(36) {
                    None => device.device_name,
                    Some((idx, _)) => device.device_name[..idx].to_string() + "...",
                }
                .cell(),
                device.sysfs_busid.cell(),
                match device.kernel_driver.as_str() {
                    "Unknown" => t!("unknown")
                        .to_string()
                        .cell()
                        .foreground_color(Some(Color::Yellow)),
                    _ => device.kernel_driver.cell(),
                },
                match device.enabled {
                    Some(t) => {
                        if t {
                            t!("enabled_yes")
                                .cell()
                                .foreground_color(Some(Color::Green))
                        } else {
                            t!("enabled_no").cell().foreground_color(Some(Color::Red))
                        }
                    }
                    None => t!("enabled_na").cell(),
                },
            ];
            table_struct.push(cell_table);
        }
        let table = table_struct
            .table()
            .title(vec![
                t!("pci_table_vendor").cell().bold(true),
                t!("pci_table_name").cell().bold(true),
                t!("pci_table_sysfs_bus_id").cell().bold(true),
                t!("pci_table_driver").cell().bold(true),
                t!("pci_table_enabled").cell().bold(true),
            ])
            .bold(true);

        let table_display = table.display().unwrap();

        println!(
            "{}\n{}",
            t!("pci_class_name_".to_string() + &class).bright_green(),
            table_display
        );
    }
}

fn display_pci_profiles_print_cli_table(target: &CfhdbPciDevice) {
    let mut table_struct = vec![];
    let mut profiles = match target.available_profiles.0.borrow().clone() {
        Some(t) => t,
        None => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("no_profiles_available_for_device")
            );
            exit(1);
        }
    };
    profiles.sort_by_key(|k| k.priority);
    for profile in profiles {
        let profile = profile.deref().clone();
        let cell_table = vec![
            profile.codename.cell(),
            match profile.i18n_desc.char_indices().nth(36) {
                None => profile.i18n_desc,
                Some((idx, _)) => profile.i18n_desc[..idx].to_string() + "...",
            }
            .cell(),
            profile.priority.cell(),
            if profile.experimental {
                t!("enabled_yes").cell().foreground_color(Some(Color::Red))
            } else {
                t!("enabled_no").cell().foreground_color(Some(Color::Green))
            },
        ];
        table_struct.push(cell_table);
    }
    let table = table_struct
        .table()
        .title(vec![
            t!("pci_table_profile_codename").cell().bold(true),
            t!("pci_table_name_i18n_desc").cell().bold(true),
            t!("pci_table_name_priority").cell().bold(true),
            t!("pci_table_name_experimental").cell().bold(true),
        ])
        .bold(true);

    let table_display = table.display().unwrap();

    println!("{}\n{}", target.sysfs_busid.bright_green(), table_display);
}

pub fn display_pci_devices(json: bool) {
    match CfhdbPciDevice::get_devices() {
        Some(devices) => {
            let profiles = match get_pci_profiles_from_url() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
            for i in &devices {
                CfhdbPciDevice::set_available_profiles(&profiles, &i);
            }
            let hashmap = CfhdbPciDevice::create_class_hashmap(devices);
            if json {
                display_pci_devices_print_json(hashmap)
            } else {
                display_pci_devices_print_cli_table(hashmap)
            }
        }
        None => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("failed_to_get_pci_devices")
            );
            exit(1);
        }
    }
}

pub fn display_pci_profiles(json: bool, target: &str) {
    match CfhdbPciDevice::get_devices() {
        Some(devices) => {
            let profiles = match get_pci_profiles_from_url() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
            let target_device = match devices.iter().find(|x| x.sysfs_busid == target) {
                Some(t) => t,
                None => {
                    eprintln!("{}", t!("device_not_found"));
                    exit(1);
                }
            };
            CfhdbPciDevice::set_available_profiles(&profiles, target_device);
            if json {
                let mut profile_rc = match target_device.available_profiles.0.borrow().clone() {
                    Some(t) => t,
                    None => {
                        eprintln!(
                            "[{}] {}",
                            t!("error").red(),
                            t!("no_profiles_available_for_device")
                        );
                        exit(1);
                    }
                };
                profile_rc.sort_by_key(|k| k.priority);
                let profiles = profile_rc.iter().map(|s| s.codename.clone()).collect::<Vec<_>>();
                let json_pretty = serde_json::to_string_pretty(&profiles)
                    .unwrap();
                println!("{}", json_pretty);
            } else {
                display_pci_profiles_print_cli_table(target_device);
            }
        }
        None => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("failed_to_get_pci_devices")
            );
            exit(1);
        }
    }
}

pub fn install_pci_profile(profile_codename: &str) {
    todo!()
}
pub fn uninstall_pci_profile(profile_codename: &str) {
    todo!()
}
pub fn enable_pci_device(target_sysfs_id: &str) {
    todo!()
}
pub fn disable_pci_device(target_sysfs_id: &str) {
    todo!()
}
pub fn start_pci_device(target_sysfs_id: &str) {
    todo!()
}
pub fn stop_pci_device(target_sysfs_id: &str) {
    todo!()
}

fn get_pci_profiles_from_url() -> Result<Vec<CfhdbPciProfile>, std::io::Error> {
    let cached_db_path = std::path::Path::new("/var/cache/cfhdb/pci.json");
    println!(
        "[{}] {}",
        t!("info").bright_green(),
        t!("pci_download_starting")
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(PCI_PROFILE_JSON_URL).send() {
        Ok(t) => {
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("pci_download_successful")
            );
            let cache = t.text().unwrap();
            let _ = std::fs::File::create(cached_db_path);
            let _ = fs::write(cached_db_path, &cache);
            cache
        }
        Err(_) => {
            println!(
                "[{}] {}",
                t!("warn").bright_yellow(),
                t!("pci_download_failed")
            );
            if cached_db_path.exists() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("pci_download_cache_found")
                );
                std::fs::read_to_string(cached_db_path).unwrap()
            } else {
                eprintln!(
                    "[{}] {}",
                    t!("error").red(),
                    t!("pci_download_cache_not_found")
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("pci_download_cache_not_found"),
                ));
            }
        }
    };
    let mut profiles_array = vec![];
    let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    if let serde_json::Value::Array(profiles) = &res["profiles"] {
        for profile in profiles {
            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let i18n_desc =
                match profile[format!("i18n_desc[{}]", rust_i18n::locale().to_string())].as_str() {
                    Some(t) => {
                        if !t.is_empty() {
                            t.to_string()
                        } else {
                            profile["i18n_desc"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string()
                        }
                    }
                    None => profile["i18n_desc"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                };
            let icon_name = profile["icon_name"]
                .as_str()
                .unwrap_or("package-x-generic")
                .to_string();
            let class_ids: Vec<String> = profile["class_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let vendor_ids: Vec<String> = profile["vendor_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let device_ids: Vec<String> = profile["device_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_class_ids: Vec<String> = profile["blacklisted_class_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_vendor_ids: Vec<String> = profile["blacklisted_vendor_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_device_ids: Vec<String> = profile["blacklisted_device_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(
                    profile["packages"]
                        .as_array()
                        .expect("invalid_usb_profile_class_ids")
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                ),
            };
            let install_script_value = profile["install_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let install_script = match install_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(install_script_value),
            };
            let remove_script_value = profile["remove_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let remove_script = match remove_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(remove_script_value),
            };
            let experimental = profile["experimental"].as_bool().unwrap_or_default();
            let removable = profile["removable"].as_bool().unwrap_or_default();
            let priority = profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbPciProfile {
                codename,
                i18n_desc,
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
            profiles_array.push(profile_struct);
            profiles_array.sort_by_key(|x| x.priority);
        }
    }
    Ok(profiles_array)
}
