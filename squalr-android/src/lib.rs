use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

// On a rooted device, the unprivileged GUI must spawn a privileged CLI app, so it is bundled into the GUI.
static SQUALR_CLI: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/../../../squalr-cli"));

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    if let Err(err) = slint::android::init(app) {
        log::error!("Failed to initialize Slint Android: {}", err);
        return;
    }

    if let Err(err) = unpack_cli() {
        log::error!("Fatal error unpacking privileged cli: {}", err);
        return;
    }

    // Create an unprivileged engine host (on android, the privileged engine is spawned as a new process).
    let mut squalr_engine = match SqualrEngine::new(EngineMode::UnprivilegedHost) {
        Ok(squalr_engine) => squalr_engine,
        Err(err) => panic!("Fatal error initializing Squalr engine: {}", err),
    };

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = match MainWindowViewModel::new(squalr_engine.get_dependency_container_mut()) {
        Ok(main_window_view) => main_window_view,
        Err(err) => panic!("Fatal error creating Squalr GUI: {}", err),
    };

    // Now that gui dependencies are registered, start the engine fully.
    squalr_engine.initialize();

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(err) => {
            log::error!("Fatal error starting Squalr: {}", err);
        }
    }
}

fn unpack_cli() -> std::io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    log::info!("Removing existing cli...");

    let _ = Command::new("su")
        .arg("-c")
        .arg("rm /data/data/rust.squalr_android/files/squalr-cli")
        .status()?;

    log::info!("Unpacking server (privileged worker)...");

    let mut child = Command::new("su")
        .arg("-c")
        .arg("cat > /data/data/rust.squalr_android/files/squalr-cli")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(SQUALR_CLI)?;
        // Closing stdin by dropping it so `cat` sees EOF:
        drop(stdin);
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to write squalr-cli via cat"));
    }

    log::info!("Elevating worker file privileges...");

    let status = Command::new("su")
        .arg("-c")
        .arg("chmod 755 /data/data/rust.squalr_android/files/squalr-cli")
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to chmod squalr-cli"));
    }

    Ok(())
}
