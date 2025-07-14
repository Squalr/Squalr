fn main() {
    // Compile user interface .slint files into usable Rust code.
    slint_build::compile("ui/slint/build.slint").unwrap();

    if cfg!(target_os = "windows") {
        // Embed windows app icon.
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../olorin/ui/images/window/app_icon.ico")
            .set("Olorin Installer", "olorin_installer.exe")
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        let _ = res.compile();
    }
}
