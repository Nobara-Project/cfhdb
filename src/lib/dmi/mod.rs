use serde::{Serialize, Serializer};
use std::{
    fs::{self},
    io::{self, ErrorKind, Write},
    os::unix::fs::PermissionsExt,
    sync::{Arc, Mutex},
};

// Implement Serialize for Arc<Mutex<Option<Vec<Arc<CfhdbDmiProfile>>>>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Arc<Mutex<Option<Vec<Arc<CfhdbDmiProfile>>>>>);
impl Serialize for ProfileWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Borrow the Mutex
        let borrowed = self.0.lock().unwrap();

        // Handle the Option
        if let Some(profiles) = &*borrowed {
            let simplified: Vec<String> =
                profiles.iter().map(|rc| rc.codename.to_string()).collect();
            simplified.serialize(serializer)
        } else {
            // Serialize as null if the Option is None
            serializer.serialize_none()
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct CfhdbDmiInfo {
    // BIOS
    pub bios_date: String,
    pub bios_release: String,
    pub bios_vendor: String,
    pub bios_version: String,
    // BOARD
    pub board_asset_tag: String,
    pub board_name: String,
    pub board_vendor: String,
    pub board_version: String,
    // PRODUCT
    pub product_family: String,
    pub product_name: String,
    pub product_sku: String,
    pub product_version: String,
    // Sys
    pub sys_vendor: String,
    // Cfhdb Extras
    pub available_profiles: ProfileWrapper,
}

impl CfhdbDmiInfo {
    fn get_dmi_string(string: &str) -> Option<String> {
        let dmi_string_path = format!("/sys/class/dmi/id/{}", string);
        match fs::read_to_string(dmi_string_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    return None;
                } else {
                    return Some(content.trim().to_owned());
                }
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn set_available_profiles(profile_data: &[CfhdbDmiProfile], info: &Self) {
        let mut available_profiles: Vec<Arc<CfhdbDmiProfile>> = vec![];
        for profile in profile_data.iter() {
            let matching = {
                if
                // BIOS
                profile.blacklisted_bios_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_bios_vendors.contains(&info.bios_vendor)
                    // BOARD
                    || profile.blacklisted_board_asset_tags.contains(&"*".to_owned())
                    || profile.blacklisted_board_asset_tags.contains(&info.board_asset_tag)
                    || profile.blacklisted_board_names.contains(&"*".to_owned())
                    || profile.blacklisted_board_names.contains(&info.board_name)
                    || profile.blacklisted_board_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_board_vendors.contains(&info.board_vendor)
                    // PRODUCT
                    || profile.blacklisted_product_families.contains(&"*".to_owned())
                    || profile.blacklisted_product_families.contains(&info.product_family)
                    || profile.blacklisted_product_names.contains(&"*".to_owned())
                    || profile.blacklisted_product_names.contains(&info.product_name)
                    || profile.blacklisted_product_skus.contains(&"*".to_owned())
                    || profile.blacklisted_product_skus.contains(&info.product_sku)
                    // Sys
                    || profile.blacklisted_sys_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_sys_vendors.contains(&info.sys_vendor)
                {
                    false
                } else {
                    let mut result = true;
                    for (profile_field, info_field) in [
                        (&profile.bios_vendors, &info.bios_vendor),
                        (&profile.board_asset_tags, &info.board_asset_tag),
                        (&profile.board_names, &info.board_name),
                        (&profile.board_vendors, &info.board_vendor),
                        (&profile.product_families, &info.product_family),
                        (&profile.product_names, &info.product_name),
                        (&profile.product_skus, &info.product_sku),
                        (&profile.sys_vendors, &info.sys_vendor),
                    ] {
                        if profile_field.contains(&"*".to_owned())
                            || profile_field.contains(info_field)
                        {
                            continue;
                        } else {
                            result = false;
                            break;
                        }
                    }
                    result
                }
            };

            if matching {
                available_profiles.push(Arc::new(profile.clone()));
            };

            if !available_profiles.is_empty() {
                *info.available_profiles.0.lock().unwrap() = Some(available_profiles.clone());
            };
        }
    }

    pub fn get_dmi() -> Self {
        let dmi = Self {
            bios_date: Self::get_dmi_string("bios_date").unwrap_or("Unknown!".to_owned()),
            bios_release: Self::get_dmi_string("bios_release").unwrap_or("Unknown!".to_owned()),
            bios_vendor: Self::get_dmi_string("bios_vendor").unwrap_or("Unknown!".to_owned()),
            bios_version: Self::get_dmi_string("bios_version").unwrap_or("Unknown!".to_owned()),
            board_asset_tag: Self::get_dmi_string("board_asset_tag")
                .unwrap_or("Unknown!".to_owned()),
            board_name: Self::get_dmi_string("board_name").unwrap_or("Unknown!".to_owned()),
            board_vendor: Self::get_dmi_string("board_vendor").unwrap_or("Unknown!".to_owned()),
            board_version: Self::get_dmi_string("board_version").unwrap_or("Unknown!".to_owned()),
            product_family: Self::get_dmi_string("product_family").unwrap_or("Unknown!".to_owned()),
            product_name: Self::get_dmi_string("product_name").unwrap_or("Unknown!".to_owned()),
            product_sku: Self::get_dmi_string("product_sku").unwrap_or("Unknown!".to_owned()),
            product_version: Self::get_dmi_string("product_version")
                .unwrap_or("Unknown!".to_owned()),
            sys_vendor: Self::get_dmi_string("sys_vendor").unwrap_or("Unknown!".to_owned()),
            available_profiles: ProfileWrapper(Arc::default()),
        };
        dmi
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CfhdbDmiProfile {
    pub codename: String,
    pub i18n_desc: String,
    pub icon_name: String,
    pub license: String,
    // BIOS
    pub bios_vendors: Vec<String>,
    // BOARD
    pub board_asset_tags: Vec<String>,
    pub board_names: Vec<String>,
    pub board_vendors: Vec<String>,
    // PRODUCT
    pub product_families: Vec<String>,
    pub product_names: Vec<String>,
    pub product_skus: Vec<String>,
    // Sys
    pub sys_vendors: Vec<String>,
    // Blacklists
    // BIOS
    pub blacklisted_bios_vendors: Vec<String>,
    // BOARD
    pub blacklisted_board_asset_tags: Vec<String>,
    pub blacklisted_board_names: Vec<String>,
    pub blacklisted_board_vendors: Vec<String>,
    // PRODUCT
    pub blacklisted_product_families: Vec<String>,
    pub blacklisted_product_names: Vec<String>,
    pub blacklisted_product_skus: Vec<String>,
    // Sys
    pub blacklisted_sys_vendors: Vec<String>,
    //
    pub packages: Option<Vec<String>>,
    pub check_script: String,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub veiled: bool,
    pub priority: i32,
}

impl CfhdbDmiProfile {
    pub fn get_profile_from_codename(
        codename: &str,
        profiles: Vec<CfhdbDmiProfile>,
    ) -> Result<Self, io::Error> {
        match profiles.iter().find(|x| x.codename == codename) {
            Some(profile) => Ok(profile.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no dmi profile with matching codename",
            )),
        }
    }

    pub fn get_status(&self) -> bool {
        let file_path = "/var/cache/cfhdb/check_cmd.sh";
        {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)
                .expect(&(file_path.to_string() + "cannot be read"));
            file.write_all(format!("#! /bin/bash\nset -e\n{}", self.check_script).as_bytes())
                .expect(&(file_path.to_string() + "cannot be written to"));
            let mut perms = file
                .metadata()
                .expect(&(file_path.to_string() + "cannot be read"))
                .permissions();
            perms.set_mode(0o777);
            fs::set_permissions(file_path, perms)
                .expect(&(file_path.to_string() + "cannot be written to"));
        }
        duct::cmd!("bash", "-c", file_path)
            .stderr_to_stdout()
            .stdout_null()
            .run()
            .is_ok()
    }
}
