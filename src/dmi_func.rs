use crate::{config::*, get_profile_url_config, run_in_lock_script};
use cli_table::{Cell, Color, Style, Table};
use colored::Colorize;
use lazy_static::lazy_static;
use libcfhdb::dmi::*;
use std::{fs, ops::Deref, path::Path, process::exit, sync::Arc};

lazy_static! {
    static ref DMI_PROFILE_JSON_URL: String = get_profile_url_config().dmi_json_url;
}

fn display_dmi_info_print_json(dmi: &CfhdbDmiInfo) {
    let json_pretty = serde_json::to_string_pretty(&dmi).unwrap();
    println!("{}", json_pretty);
}
fn display_dmi_info_print_cli_table(dmi: &CfhdbDmiInfo) {
    let mut table_struct = vec![];
    for (dmi_string, dmi_value) in [
        (t!("dmi_bios_date_string"), &dmi.bios_date),
        (t!("dmi_bios_release_string"), &dmi.bios_release),
        (t!("dmi_bios_vendor_string"), &dmi.bios_vendor),
        (t!("dmi_bios_version_string"), &dmi.bios_version),
        // BOARD
        (t!("dmi_board_asset_tag_string"), &dmi.board_asset_tag),
        (t!("dmi_board_name_string"), &dmi.board_name),
        (t!("dmi_board_vendor_string"), &dmi.board_vendor),
        (t!("dmi_board_version_string"), &dmi.board_version),
        // PRODUCT
        (t!("dmi_product_family_string"), &dmi.product_family),
        (t!("dmi_product_name_string"), &dmi.product_name),
        (t!("dmi_product_sku_string"), &dmi.product_sku),
        (t!("dmi_product_version_string"), &dmi.product_version),
        // Sys
        (t!("dmi_sys_vendor_string"), &dmi.sys_vendor),
    ] {
        let cell_table = vec![
            dmi_string.cell(),
            match dmi_value.as_str() {
                "Unknown!" => dmi_value.cell().foreground_color(Some(Color::Yellow)),
                _ => dmi_value.cell().foreground_color(Some(Color::Green)),
            },
        ];
        table_struct.push(cell_table);
    }
    let table = table_struct
        .table()
        .title(vec![
            t!("dmi_table_string").cell().bold(true),
            t!("dmi_table_value").cell().bold(true),
        ])
        .bold(true);

    let table_display = table.display().unwrap();

    println!(
        "{}\n{}",
        t!("dmi_info_header").bright_green(),
        table_display
    );
}

fn display_dmi_profiles_print_cli_table(target: &CfhdbDmiInfo) {
    let mut table_struct = vec![];
    let mut profiles = match target.available_profiles.0.lock().unwrap().clone() {
        Some(t) => t,
        None => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("no_profiles_available_for_info")
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
            t!("table_profile_codename").cell().bold(true),
            t!("table_name_i18n_desc").cell().bold(true),
            t!("table_name_license").cell().bold(true),
            t!("table_name_priority").cell().bold(true),
            t!("table_name_experimental").cell().bold(true),
            t!("table_name_installed").cell().bold(true),
        ])
        .bold(true);

    let table_display = table.display().unwrap();

    println!("{}", table_display);
}

pub fn display_dmi_info(json: bool) {
    let dmi = CfhdbDmiInfo::get_dmi();
    let profiles = match get_dmi_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    CfhdbDmiInfo::set_available_profiles(&profiles, &dmi);
    if json {
        display_dmi_info_print_json(&dmi)
    } else {
        display_dmi_info_print_cli_table(&dmi)
    }
}

pub fn display_dmi_profiles(json: bool) {
    let dmi_info = CfhdbDmiInfo::get_dmi();
    let profiles = match get_dmi_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    CfhdbDmiInfo::set_available_profiles(&profiles, &dmi_info);
    if json {
        let mut profile_arc = match dmi_info.available_profiles.0.lock().unwrap().clone() {
            Some(t) => t,
            None => {
                eprintln!(
                    "[{}] {}",
                    t!("error").red(),
                    t!("no_profiles_available_for_info")
                );
                exit(1);
            }
        };
        profile_arc.sort_by_key(|k| k.priority);
        let profiles = profile_arc
            .iter()
            .map(|s| s.codename.clone())
            .collect::<Vec<_>>();
        let json_pretty = serde_json::to_string_pretty(&profiles).unwrap();
        println!("{}", json_pretty);
    } else {
        display_dmi_profiles_print_cli_table(&dmi_info);
    }
}

pub fn install_dmi_profile(profile_codename: &str) {
    let profiles = match get_dmi_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    match CfhdbDmiProfile::get_profile_from_codename(profile_codename, profiles) {
        Ok(target_profile) => {
            if target_profile.get_status() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("profile_already_installed")
                );
            } else {
                match target_profile.install_script {
                    Some(t) => match target_profile.packages {
                        Some(a) => {
                            let package_list = a.join(" ");
                            run_in_lock_script(&format!(
                                "#! /bin/bash\nset -e\n{}\n{}",
                                distro_packages_installer(&package_list),
                                t
                            ));
                        }
                        None => {
                            run_in_lock_script(&format!("#! /bin/bash\nset -e\n{}", t));
                        }
                    },
                    None => match target_profile.packages {
                        Some(a) => {
                            let package_list = a.join(" ");
                            run_in_lock_script(&format!(
                                "#! /bin/bash\nset -e\n{}",
                                distro_packages_installer(&package_list)
                            ));
                        }
                        None => {}
                    },
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
pub fn uninstall_dmi_profile(profile_codename: &str) {
    let profiles = match get_dmi_profiles_from_url() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] {}", t!("error").red(), e);
            exit(1);
        }
    };
    match CfhdbDmiProfile::get_profile_from_codename(profile_codename, profiles) {
        Ok(target_profile) => {
            if !target_profile.get_status() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("profile_not_installed")
                );
            } else {
                match target_profile.remove_script {
                    Some(t) => match target_profile.packages {
                        Some(a) => {
                            let package_list = a.join(" ");
                            run_in_lock_script(&format!(
                                "#! /bin/bash\nset -e\n{}\n{}",
                                distro_packages_uninstaller(&package_list),
                                t
                            ));
                        }
                        None => {
                            run_in_lock_script(&format!("#! /bin/bash\nset -e\n{}", t));
                        }
                    },
                    None => match target_profile.packages {
                        Some(a) => {
                            let package_list = a.join(" ");
                            run_in_lock_script(&format!(
                                "#! /bin/bash\nset -e\n{}",
                                distro_packages_uninstaller(&package_list)
                            ));
                        }
                        None => {}
                    },
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

fn get_dmi_profiles_from_url() -> Result<Vec<CfhdbDmiProfile>, std::io::Error> {
    let cached_db_path = Path::new("/var/cache/cfhdb/dmi.json");
    println!(
        "[{}] {}",
        t!("info").bright_green(),
        t!("dmi_download_starting")
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(DMI_PROFILE_JSON_URL.clone()).send() {
        Ok(t) => {
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("dmi_download_successful")
            );
            let cache = t.text().unwrap();
            let _ = fs::File::create(cached_db_path);
            let _ = fs::write(cached_db_path, &cache);
            cache
        }
        Err(_) => {
            println!(
                "[{}] {}",
                t!("warn").bright_yellow(),
                t!("dmi_download_failed")
            );
            if cached_db_path.exists() {
                println!(
                    "[{}] {}",
                    t!("info").bright_green(),
                    t!("dmi_download_cache_found")
                );
                fs::read_to_string(cached_db_path).unwrap()
            } else {
                eprintln!(
                    "[{}] {}",
                    t!("error").red(),
                    t!("dmi_download_cache_not_found")
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("dmi_download_cache_not_found"),
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
            let mut dmi_strings_vec = Vec::new();
            for dmi_string in [
                "bios_vendors",
                "board_asset_tags",
                "board_names",
                "board_vendors",
                "product_families",
                "product_names",
                "product_skus",
                "sys_vendors",
                "blacklisted_bios_vendors",
                "blacklisted_board_asset_tags",
                "blacklisted_board_names",
                "blacklisted_board_vendors",
                "blacklisted_product_families",
                "blacklisted_product_names",
                "blacklisted_product_skus",
                "blacklisted_sys_vendors",
            ] {
                let final_map: Vec<String> = match profile[dmi_string].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
                dmi_strings_vec.push(Arc::new(final_map));
            }
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(
                    profile["packages"]
                        .as_array()
                        .expect("invalid_dmi_profile_json")
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
            let veiled = profile["veiled"].as_bool().unwrap_or_default();
            let priority = profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbDmiProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                bios_vendors: dmi_strings_vec[0].to_vec(),
                board_asset_tags: dmi_strings_vec[1].to_vec(),
                board_names: dmi_strings_vec[2].to_vec(),
                board_vendors: dmi_strings_vec[3].to_vec(),
                product_families: dmi_strings_vec[4].to_vec(),
                product_names: dmi_strings_vec[5].to_vec(),
                product_skus: dmi_strings_vec[6].to_vec(),
                sys_vendors: dmi_strings_vec[7].to_vec(),
                blacklisted_bios_vendors: dmi_strings_vec[8].to_vec(),
                blacklisted_board_asset_tags: dmi_strings_vec[9].to_vec(),
                blacklisted_board_names: dmi_strings_vec[10].to_vec(),
                blacklisted_board_vendors: dmi_strings_vec[11].to_vec(),
                blacklisted_product_families: dmi_strings_vec[12].to_vec(),
                blacklisted_product_names: dmi_strings_vec[13].to_vec(),
                blacklisted_product_skus: dmi_strings_vec[14].to_vec(),
                blacklisted_sys_vendors: dmi_strings_vec[15].to_vec(),
                packages,
                check_script,
                install_script,
                remove_script,
                experimental,
                removable,
                veiled,
                priority: priority as i32,
            };
            profiles_array.push(profile_struct);
            profiles_array.sort_by_key(|x| x.priority);
        }
    }
    Ok(profiles_array)
}
