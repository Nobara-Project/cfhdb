pub fn distro_packages_installer(package_list: &str) -> String {
    format!("sudo dnf install {}", package_list)
}

pub fn distro_packages_uninstaller(package_list: &str) -> String {
    format!("sudo dnf remove {}", package_list)
}
