pub const PCI_PROFILE_JSON_URL: &str =
    "https://github.com/CosmicFusion/cfhdb/raw/refs/heads/master/data/profiles/pci.json";
pub const USB_PROFILE_JSON_URL: &str =
    "https://github.com/CosmicFusion/cfhdb/raw/refs/heads/master/data/profiles/usb.json";

pub fn distro_packages_installer(package_list: &str) -> String {
    format!("pikman install {}", package_list)
}

pub fn distro_packages_uninstaller(package_list: &str) -> String {
    format!("pikman purge {}", package_list)
}
