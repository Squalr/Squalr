use squalr_engine::squalr_engine::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

// On a rooted device, the unprivileged GUI must spawn a privileged CLI app, so it is bundled into the GUI.
static SQUALR_CLI: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/../../../squalr-cli"));

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = MainWindowViewModel::new();

    if let Err(err) = unpack_cli() {
        Logger::get_instance().log(LogLevel::Error, "Fatal error unpacking privileged cli.", Some(err.to_string().as_str()));
        return;
    }

    SqualrEngine::initialize(EngineMode::UnprivilegedHost);

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(err) => {
            Logger::get_instance().log(LogLevel::Error, "Fatal error starting Squalr.", Some(err.to_string().as_str()));
        }
    }
}

fn unpack_cli() -> std::io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    Logger::get_instance().log(LogLevel::Info, "Removing existing cli...", None);

    let _ = Command::new("su")
        .arg("-c")
        .arg("rm /data/data/rust.squalr_android/files/squalr-cli")
        .status()?;

    Logger::get_instance().log(LogLevel::Info, "Unpacking server (privileged worker)...", None);

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

    Logger::get_instance().log(LogLevel::Info, "Elevating worker file privileges...", None);

    let status = Command::new("su")
        .arg("-c")
        .arg("chmod 755 /data/data/rust.squalr_android/files/squalr-cli")
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to chmod squalr-cli"));
    }

    Ok(())
}
