use crate::commands::engine_command::EngineCommand;
use crate::commands::inter_process_command_pipe::InterProcessCommandPipe;
use interprocess::local_socket::prelude::LocalSocketStream;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;

#[cfg(any(target_os = "android"))]
static SQUALR_CLI: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/squalr-cli"));

pub struct InterProcessCommandDispatcher {
    ipc_server: Arc<RwLock<Option<Child>>>,
    ipc_connection: Arc<RwLock<Option<LocalSocketStream>>>,
}

impl InterProcessCommandDispatcher {
    pub fn new() -> InterProcessCommandDispatcher {
        let instance = InterProcessCommandDispatcher {
            ipc_server: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance.initialize();
        instance
    }

    fn initialize(&self) {
        Logger::get_instance().log(LogLevel::Info, "Spawning squalr-cli privileged shell...", None);

        let ipc_server = self.ipc_server.clone();
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            match Self::spawn_squalr_cli_as_root() {
                Ok(child) => {
                    Logger::get_instance().log(LogLevel::Info, "Spawned squalr-cli as root.", None);

                    // Update the server handle
                    if let Ok(mut server) = ipc_server.write() {
                        *server = Some(child);
                    }

                    match InterProcessCommandPipe::create_client() {
                        Ok(stream) => {
                            if let Ok(mut ipc_connection) = ipc_connection.write() {
                                *ipc_connection = Some(stream);
                            }
                        }
                        Err(err) => {
                            Logger::get_instance().log(LogLevel::Error, &format!("Error creating IPC manager: {}", err), None);
                        }
                    }
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to spawn squalr-cli as root: {}", err), None);
                }
            }
        });
    }

    pub fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        if let Err(err) = InterProcessCommandPipe::ipc_send(&self.ipc_connection, command) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC command: {}", err), None);
        }
    }

    #[cfg(any(target_os = "android"))]
    fn spawn_squalr_cli_as_root() -> std::io::Result<std::process::Child> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        Logger::get_instance().log(LogLevel::Info, "Removing existing cli...", None);

        let status = Command::new("su")
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

        Logger::get_instance().log(LogLevel::Info, "Spawning privileged worker...", None);

        let child = Command::new("su")
            .arg("-c")
            .arg("/data/data/rust.squalr_android/files/squalr-cli --ipc-mode")
            .spawn()?;

        Ok(child)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("sudo").arg("squalr-cli").arg("--ipc-mode").spawn()
    }

    #[cfg(windows)]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        // No actual privilege escallation for windows -- this feature is not supposed to be used on windows at all.
        // So, just spawn it normally for the rare occasion that we are testing this feature on windows.
        Command::new("squalr-cli").arg("--ipc-mode").spawn()
    }
}
