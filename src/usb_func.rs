use crate::config::*;
use cli_table::{format::Justify, Cell, Color, Style, Table};
use colored::Colorize;
use libcfhdb::usb::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;
use users::get_current_username;

fn display_usb_devices_print_json(hashmap: HashMap<String, Vec<CfhdbUsbDevice>>) {
    let json_pretty = serde_json::to_string_pretty(&hashmap).unwrap();
    println!("{}", json_pretty);
}
fn display_usb_devices_print_cli_table(hashmap: HashMap<String, Vec<CfhdbUsbDevice>>) {
    for (class, devices) in hashmap {
        let mut table_struct = vec![];
        for device in devices {
            let cell_table = vec![
                match device.manufacturer_string_index.char_indices().nth(18) {
                    None => device.manufacturer_string_index,
                    Some((idx, _)) => device.manufacturer_string_index[..idx].to_string() + "...",
                }
                .cell(),
                match device.product_string_index.char_indices().nth(36) {
                    None => device.product_string_index,
                    Some((idx, _)) => device.product_string_index[..idx].to_string() + "...",
                }
                .cell(),
                device.sysfs_busid.cell(),
                device.speed.cell(),
                match device.kernel_driver.as_str() {
                    "Unknown" => t!("unknown")
                        .to_string()
                        .cell()
                        .foreground_color(Some(Color::Yellow)),
                    _ => device.kernel_driver.cell(),
                },
                match device.started {
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
                if device.enabled {
                    t!("enabled_yes")
                        .cell()
                        .foreground_color(Some(Color::Green))
                } else {
                    t!("enabled_no").cell().foreground_color(Some(Color::Red))
                },
            ];
            table_struct.push(cell_table);
        }
        let table = table_struct
            .table()
            .title(vec![
                t!("usb_table_manufacturer_string_index").cell().bold(true),
                t!("usb_table_product_string_index").cell().bold(true),
                t!("usb_table_sysfs_bus_id").cell().bold(true),
                t!("usb_table_speed").cell().bold(true),
                t!("usb_table_driver").cell().bold(true),
                t!("usb_table_started").cell().bold(true),
                t!("usb_table_enabled").cell().bold(true),
            ])
            .bold(true);

        let table_display = table.display().unwrap();

        println!(
            "{}\n{}",
            t!("usb_class_name_".to_string() + &class).bright_green(),
            table_display
        );
    }
}

fn display_usb_profiles_print_cli_table(target: &CfhdbUsbDevice) {
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
        let profile_status = profile.get_status();
        let cell_table = vec![
            profile.codename.cell(),
            match profile.i18n_desc.char_indices().nth(36) {
                None => profile.i18n_desc,
                Some((idx, _)) => profile.i18n_desc[..idx].to_string() + "...",
            }
            .cell(),
            profile.license.cell(),
            profile.priority.cell(),
            if profile.experimental {
                t!("enabled_yes").cell().foreground_color(Some(Color::Red))
            } else {
                t!("enabled_no").cell().foreground_color(Some(Color::Green))
            },
            if profile_status {
                t!("enabled_yes")
                    .cell()
                    .foreground_color(Some(Color::Green))
            } else {
                t!("enabled_no").cell().foreground_color(Some(Color::Red))
            },
        ];
        table_struct.push(cell_table);
    }
    let table = table_struct
        .table()
        .title(vec![
            t!("usb_table_profile_codename").cell().bold(true),
            t!("usb_table_name_i18n_desc").cell().bold(true),
            t!("usb_table_name_license").cell().bold(true),
            t!("usb_table_name_priority").cell().bold(true),
            t!("usb_table_name_experimental").cell().bold(true),
            t!("usb_table_name_installed").cell().bold(true),
        ])
        .bold(true);

    let table_display = table.display().unwrap();

    println!("{}\n{}", target.sysfs_busid.bright_green(), table_display);
}

pub fn display_usb_devices(json: bool) {
    match CfhdbUsbDevice::get_devices() {
        Some(devices) => {
            let profiles = match get_usb_profiles_from_url() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
            for i in &devices {
                CfhdbUsbDevice::set_available_profiles(&profiles, &i);
            }
            let hashmap = CfhdbUsbDevice::create_class_hashmap(devices);
            if json {
                display_usb_devices_print_json(hashmap)
            } else {
                display_usb_devices_print_cli_table(hashmap)
            }
        }
        None => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("failed_to_get_usb_devices")
            );
            exit(1);
        }
    }
}

pub fn display_usb_profiles(json: bool, target: &str) {
    match CfhdbUsbDevice::get_device_from_busid(target) {
        Ok(target_device) => {
            let profiles = match get_usb_profiles_from_url() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
            CfhdbUsbDevice::set_available_profiles(&profiles, &target_device);
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
                let profiles = profile_rc
                    .iter()
                    .map(|s| s.codename.clone())
                    .collect::<Vec<_>>();
                let json_pretty = serde_json::to_string_pretty(&profiles).unwrap();
                println!("{}", json_pretty);
            } else {
                display_usb_profiles_print_cli_table(&target_device);
            }
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("no_matching_usb_device"));
            exit(1);
        }
    }
}

pub fn install_usb_profile(profile_codename: &str) {
    let profiles = match get_usb_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    match CfhdbUsbProfile::get_profile_from_codename(profile_codename, profiles) {
        Ok(target_profile) => {
            if target_profile.get_status() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("profile_already_installed")
                );
            } else {
                match target_profile.packages {
                    Some(t) => {
                        let package_list = t.join(" ");
                        match duct::cmd!("pikman", "install", package_list).run() {
                            Ok(_) => {
                                println!(
                                    "[{}] {}",
                                    t!("info").bright_green(),
                                    t!("package_installation_successful")
                                );
                            }
                            Err(_) => {
                                eprintln!(
                                    "[{}] {}",
                                    t!("error").red(),
                                    t!("package_installation_failed")
                                );
                                exit(1);
                            }
                        }
                    }
                    None => {}
                }

                match target_profile.install_script {
                    Some(t) => {
                        let file_path = "/var/cache/cfhdb/script_lock.sh";
                        let file_fs_path = Path::new(file_path);
                        if file_fs_path.exists() {
                            fs::remove_file(file_fs_path).unwrap();
                        }
                        {
                            let mut file = std::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(file_path)
                                .expect(&(file_path.to_string() + "cannot be read"));
                            file.write_all(format!("#! /bin/bash\nset -e\n{}", t).as_bytes())
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
                                eprintln!(
                                    "[{}] {}",
                                    t!("error").red(),
                                    t!("install_script_failed")
                                );
                                fs::remove_file(file_fs_path).unwrap();
                                exit(1);
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        Err(_) => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("no_matching_profile_codename")
            );
            exit(1);
        }
    }
}
pub fn uninstall_usb_profile(profile_codename: &str) {
    let profiles = match get_usb_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    match CfhdbUsbProfile::get_profile_from_codename(profile_codename, profiles) {
        Ok(target_profile) => {
            if !target_profile.get_status() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("profile_not_installed")
                );
            } else {
                match target_profile.packages {
                    Some(t) => {
                        let package_list = t.join(" ");
                        match duct::cmd!("pikman", "purge", package_list).run() {
                            Ok(_) => {
                                match duct::cmd!("pikman", "purge").run() {
                                    Ok(_) => {
                                        println!(
                                            "[{}] {}",
                                            t!("info").bright_green(),
                                            t!("package_removal_successful")
                                        );
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "[{}] {}",
                                            t!("error").red(),
                                            t!("package_removal_failed")
                                        );
                                        exit(1);
                                    }
                                }
                                println!(
                                    "[{}] {}",
                                    t!("info").bright_green(),
                                    t!("package_removal_successful")
                                );
                            }
                            Err(_) => {
                                eprintln!(
                                    "[{}] {}",
                                    t!("error").red(),
                                    t!("package_removal_failed")
                                );
                                exit(1);
                            }
                        }
                    }
                    None => {}
                }

                match target_profile.remove_script {
                    Some(t) => {
                        let file_path = "/var/cache/cfhdb/script_lock.sh";
                        let file_fs_path = Path::new(file_path);
                        if file_fs_path.exists() {
                            fs::remove_file(file_fs_path).unwrap();
                        }
                        {
                            let mut file = std::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(file_path)
                                .expect(&(file_path.to_string() + "cannot be read"));
                            file.write_all(format!("#! /bin/bash\nset -e\n{}", t).as_bytes())
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
                                    t!("remove_script_successful")
                                );
                                fs::remove_file(file_fs_path).unwrap();
                            }
                            Err(_) => {
                                eprintln!("[{}] {}", t!("error").red(), t!("remove_script_failed"));
                                fs::remove_file(file_fs_path).unwrap();
                                exit(1);
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        Err(_) => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("no_matching_profile_codename")
            );
            exit(1);
        }
    }
}

pub fn enable_usb_device(target_sysfs_id: &str) {
    match CfhdbUsbDevice::get_device_from_busid(target_sysfs_id) {
        Ok(target_device) => {
            match target_device.enable_device() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("no_matching_usb_device"));
            exit(1);
        }
    }
}
pub fn disable_usb_device(target_sysfs_id: &str) {
    match CfhdbUsbDevice::get_device_from_busid(target_sysfs_id) {
        Ok(target_device) => {
            match target_device.disable_device() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("no_matching_usb_device"));
            exit(1);
        }
    }
}

pub fn start_usb_device(target_sysfs_id: &str) {
    match CfhdbUsbDevice::get_device_from_busid(target_sysfs_id) {
        Ok(target_device) => {
            match target_device.start_device() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("no_matching_usb_device"));
            exit(1);
        }
    }
}
pub fn stop_usb_device(target_sysfs_id: &str) {
    match CfhdbUsbDevice::get_device_from_busid(target_sysfs_id) {
        Ok(target_device) => {
            match target_device.stop_device() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[{}] {}", t!("error").red(), e);
                    exit(1);
                }
            };
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("no_matching_usb_device"));
            exit(1);
        }
    }
}

fn get_usb_profiles_from_url() -> Result<Vec<CfhdbUsbProfile>, std::io::Error> {
    let cached_db_path = std::path::Path::new("/var/cache/cfhdb/usb.json");
    println!(
        "[{}] {}",
        t!("info").bright_green(),
        t!("usb_download_starting")
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(USB_PROFILE_JSON_URL).send() {
        Ok(t) => {
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("usb_download_successful")
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
                t!("usb_download_failed")
            );
            if cached_db_path.exists() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("usb_download_cache_found")
                );
                std::fs::read_to_string(cached_db_path).unwrap()
            } else {
                eprintln!(
                    "[{}] {}",
                    t!("error").red(),
                    t!("usb_download_cache_not_found")
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("usb_download_cache_not_found"),
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
            let license = profile["license"]
                .as_str()
                .unwrap_or(&t!("unknown"))
                .to_string();
            let class_codes: Vec<String> = match profile["class_codes"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let vendor_ids: Vec<String> = match profile["vendor_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let product_ids: Vec<String> = match profile["product_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let blacklisted_class_codes: Vec<String> =
                match profile["blacklisted_class_codes"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_vendor_ids: Vec<String> =
                match profile["blacklisted_vendor_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_product_ids: Vec<String> =
                match profile["blacklisted_product_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
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
            let check_script = profile["check_script"]
                .as_str()
                .unwrap_or("false")
                .to_string();
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
            let profile_struct = CfhdbUsbProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                class_codes,
                vendor_ids,
                product_ids,
                blacklisted_class_codes,
                blacklisted_vendor_ids,
                blacklisted_product_ids,
                packages,
                check_script,
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
