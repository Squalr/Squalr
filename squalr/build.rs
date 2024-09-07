fn main() {
    // Compile user interface .slint files into usable Rust code.
    slint_build::compile("ui/build.slint").unwrap();

    // Embed windows app icon.
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("ui/images/app/app_icon.ico")
            .set("Squalr", "squalr.exe")
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        let _ = res.compile();
    }
}
