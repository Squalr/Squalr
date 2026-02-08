fn main() {
    if cfg!(target_os = "windows") {
        // Embed Windows metadata.
        let mut res = winresource::WindowsResource::new();
        res.set("Squalr Installer", "squalr_installer.exe")
            .set_icon("../squalr/images/app/app_icon.ico")
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        let _ = res.compile();
    }
}
