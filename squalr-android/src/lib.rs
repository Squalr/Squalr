#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: squalr::AndroidApp) {
    if let Err(error) = squalr::run_gui_android(android_app) {
        log::error!("Fatal Android GUI bootstrap failure: {error:?}");
    }
}
