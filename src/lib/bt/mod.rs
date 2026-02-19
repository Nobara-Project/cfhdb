use serde::{Serialize, Serializer};
use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind, Write},
    os::unix::fs::PermissionsExt,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

// Implement Serialize for Arc<Mutex<Option<Vec<Arc<CfhdbBtProfile>>>>>

#[derive(Debug, Clone)]
pub struct ProfileWrapper(pub Arc<Mutex<Option<Vec<Arc<CfhdbBtProfile>>>>>);
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
pub struct CfhdbBtDevice {
    // String identification
    pub alias: String,
    pub name: String,
    // Vendor IDs
    pub class_id: String,
    // modalias
    pub modalias_vendor_id: String,
    pub modalias_product_id: String,
    pub modalias_device_id: String,
    // System Info
    pub adapter: String,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
    pub blocked: bool,
    pub address: String,
    pub battery_level: u8,
    // Cfhdb Extras
    pub available_profiles: ProfileWrapper,
    // Bluer
    #[serde(skip_serializing)]
    bluer_device: bluer::Device
}

impl CfhdbBtDevice {
    pub fn set_available_profiles(profile_data: &[CfhdbBtProfile], device: &Self) {
        let mut available_profiles: Vec<Arc<CfhdbBtProfile>> = vec![];
        for profile in profile_data.iter() {
            let matching = {
                if (profile.blacklisted_class_ids.contains(&"*".to_owned())
                    || profile.blacklisted_class_ids.contains(&device.class_id))
                    || (profile.blacklisted_bt_names.contains(&"*".to_owned())
                        || profile.blacklisted_bt_names.contains(&device.name))
                    || (profile
                        .blacklisted_modalias_device_ids
                        .contains(&"*".to_owned())
                        || profile
                            .blacklisted_modalias_device_ids
                            .contains(&device.modalias_device_id))
                    || (profile
                        .blacklisted_modalias_product_ids
                        .contains(&"*".to_owned())
                        || profile
                            .blacklisted_modalias_product_ids
                            .contains(&device.modalias_product_id))
                    || (profile
                        .blacklisted_modalias_vendor_ids
                        .contains(&"*".to_owned())
                        || profile
                            .blacklisted_modalias_vendor_ids
                            .contains(&device.modalias_vendor_id))
                {
                    false
                } else {
                    let mut result = true;
                    for (profile_field, info_field) in [
                        (&profile.bt_names, &device.name),
                        (&profile.modalias_device_ids, &device.modalias_device_id),
                        (&profile.modalias_product_ids, &device.modalias_product_id),
                        (&profile.modalias_vendor_ids, &device.modalias_vendor_id),
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
                *device.available_profiles.0.lock().unwrap() = Some(available_profiles.clone());
            };
        }
    }

    pub fn disconnect_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.disconnect().await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn connect_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.connect().await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn block_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.set_blocked(true).await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn unblock_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.set_blocked(false).await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn trust_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.set_trusted(true).await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn untrust_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.set_trusted(false).await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn pair_device(&self) -> Result<(), io::Error> {
        let bluer_future = async {
            let bluer_device = &self.bluer_device;
            bluer_device.pair().await
        };
        let rt = Runtime::new()?;
        match rt.block_on(bluer_future) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::Error::new(ErrorKind::Other, "failed")),
        }
    }

    pub fn get_device_from_address(address: &str) -> Result<CfhdbBtDevice, io::Error> {
        let devices = match CfhdbBtDevice::get_devices() {
            Some(t) => t,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Could not get bt devices",
                ));
            }
        };
        match devices.iter().find(|x| x.address == address) {
            Some(device) => Ok(device.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no bt device with matching busid",
            )),
        }
    }

    fn format_bt_address(bytes: [u8; 6]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":")
    }

    //
    async fn get_devices_future() -> Result<Vec<Self>, bluer::Error> {
        // Initialize
        let session = bluer::Session::new().await?;
        let adapter_names = session.adapter_names().await?;
        let mut devices = vec![];

        for adapter_name in adapter_names {
            let adapter = session.adapter(&adapter_name)?;
            let bt_devices = adapter.device_addresses().await?;

            for addr in bt_devices {
                let device = adapter.device(addr)?;

                let device_modalias = device.modalias().await?;

                devices.push(Self {
                    alias: device.alias().await.unwrap_or("Unknown!".to_owned()),
                    name: device
                        .name()
                        .await
                        .unwrap_or(None)
                        .unwrap_or("Unknown!".to_owned()),
                    class_id: match device.class().await {
                        Ok(t) => match t {
                            Some(x) => x.to_string(),
                            None => "Unknown!".to_owned(),
                        },
                        Err(_) => "Unknown!".to_owned(),
                    },
                    modalias_device_id: match &device_modalias {
                        Some(t) => t.device.to_string(),
                        None => "Unknown!".to_owned(),
                    },
                    modalias_vendor_id: match &device_modalias {
                        Some(t) => t.vendor.to_string(),
                        None => "Unknown!".to_owned(),
                    },
                    modalias_product_id: match &device_modalias {
                        Some(t) => t.product.to_string(),
                        None => "Unknown!".to_owned(),
                    },
                    adapter: adapter_name.clone(),
                    paired: device.is_paired().await.unwrap_or_default(),
                    connected: device.is_connected().await.unwrap_or_default(),
                    trusted: device.is_trusted().await.unwrap_or_default(),
                    blocked: device.is_blocked().await.unwrap_or_default(),
                    battery_level: device.battery_percentage().await.unwrap_or_default().unwrap_or_default(),
                    address: Self::format_bt_address(addr.0),
                    bluer_device: device,
                    available_profiles: ProfileWrapper(Arc::default()),
                });
            }
        }

        Ok(devices)
    }

    pub fn get_devices() -> Option<Vec<Self>> {
        let rt = Runtime::new().unwrap();
        match rt.block_on(Self::get_devices_future()) {
            Ok(t) => return Some(t),
            Err(_) => return None,
        };
    }

    pub fn create_class_hashmap(devices: Vec<Self>) -> HashMap<String, Vec<Self>> {
        let mut map: HashMap<String, Vec<Self>> = HashMap::new();

        for device in devices {
            // Use the entry API to get or create a Vec for the key
            map.entry(device.class_id.clone())
                .or_insert_with(Vec::new)
                .push(device);
        }

        map
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CfhdbBtProfile {
    pub codename: String,
    pub i18n_desc: String,
    pub icon_name: String,
    pub license: String,
    pub class_ids: Vec<String>,
    pub bt_names: Vec<String>,
    pub modalias_vendor_ids: Vec<String>,
    pub modalias_device_ids: Vec<String>,
    pub modalias_product_ids: Vec<String>,
    pub blacklisted_class_ids: Vec<String>,
    pub blacklisted_bt_names: Vec<String>,
    pub blacklisted_modalias_vendor_ids: Vec<String>,
    pub blacklisted_modalias_device_ids: Vec<String>,
    pub blacklisted_modalias_product_ids: Vec<String>,
    pub packages: Option<Vec<String>>,
    pub check_script: String,
    pub install_script: Option<String>,
    pub remove_script: Option<String>,
    pub experimental: bool,
    pub removable: bool,
    pub veiled: bool,
    pub priority: i32,
}

impl CfhdbBtProfile {
    pub fn get_profile_from_codename(
        codename: &str,
        profiles: Vec<CfhdbBtProfile>,
    ) -> Result<Self, io::Error> {
        match profiles.iter().find(|x| x.codename == codename) {
            Some(profile) => Ok(profile.clone()),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "no bt profile with matching codename",
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
