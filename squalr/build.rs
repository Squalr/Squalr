fn main() {
    if cfg!(target_os = "windows") {
        // Embed windows app icon.
        let mut res = winresource::WindowsResource::new();
        res.set_icon("ui/images/app/app_icon.ico")
            .set("Squalr", "squalr.exe")
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        let _ = res.compile();
    }
}
