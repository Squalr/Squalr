use olorin_engine::engine_mode::EngineMode;
use olorin_engine::olorin_engine::OlorinEngine;
use olorin_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

// On a rooted device, the unprivileged GUI must spawn a privileged CLI app, so it is bundled into the GUI.
static SQUALR_CLI: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/../../../olorin-cli"));

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    if let Err(error) = slint::android::init(app) {
        log::error!("Failed to initialize Slint Android: {}", error);
        return;
    }

    if let Err(error) = unpack_cli() {
        log::error!("Fatal error unpacking privileged cli: {}", error);
        return;
    }

    // Create an unprivileged engine host (on android, the privileged engine is spawned as a new process).
    let mut olorin_engine = match OlorinEngine::new(EngineMode::UnprivilegedHost) {
        Ok(olorin_engine) => olorin_engine,
        Err(error) => panic!("Fatal error initializing Olorin engine: {}", error),
    };

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = match MainWindowViewModel::new(olorin_engine.get_dependency_container_mut()) {
        Ok(main_window_view) => main_window_view,
        Err(error) => panic!("Fatal error creating Olorin GUI: {}", error),
    };

    // Now that gui dependencies are registered, start the engine fully.
    olorin_engine.initialize();

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(error) => {
            log::error!("Fatal error starting Olorin: {}", error);
        }
    }
}

fn unpack_cli() -> std::io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    log::info!("Removing existing cli...");

    let _ = Command::new("su")
        .arg("-c")
        .arg("rm /data/data/rust.olorin_android/files/olorin-cli")
        .status()?;

    log::info!("Unpacking server (privileged worker)...");

    let mut child = Command::new("su")
        .arg("-c")
        .arg("cat > /data/data/rust.olorin_android/files/olorin-cli")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(SQUALR_CLI)?;
        // Closing stdin by dropping it so `cat` sees EOF:
        drop(stdin);
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to write olorin-cli via cat"));
    }

    log::info!("Elevating worker file privileges...");

    let status = Command::new("su")
        .arg("-c")
        .arg("chmod 755 /data/data/rust.olorin_android/files/olorin-cli")
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to chmod olorin-cli"));
    }

    Ok(())
}
