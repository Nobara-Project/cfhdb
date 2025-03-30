pub fn distro_packages_installer(package_list: &str) -> String {
    format!("pikman install {}", package_list)
}

pub fn distro_packages_uninstaller(package_list: &str) -> String {
    format!("pikman purge {}", package_list)
}
