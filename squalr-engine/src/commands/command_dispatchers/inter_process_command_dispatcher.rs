use crate::commands::engine_command::EngineCommand;
use interprocess::local_socket::ToFsName;
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::Stream;
use interprocess::os::windows::local_socket::NamedPipe;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;

const IPC_SOCKET_PATH: &str = if cfg!(windows) { "\\\\.\\pipe\\squalr-ipc" } else { "/tmp/squalr-ipc.sock" };

// Modified InterProcessCommandDispatcher
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

    pub fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        if let Err(err) = self.ipc_send_command(command) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC command: {}", err), None);
        }
    }

    pub fn ipc_send_command(
        &self,
        command: &mut EngineCommand,
    ) -> io::Result<Vec<u8>> {
        let encoded: Vec<u8> = bincode::serialize(&command).unwrap();

        if let Ok(connection) = self.ipc_connection.read() {
            if let Some(mut stream) = connection.as_ref() {
                // Write message length first (as u32)
                let len = encoded.len() as u32;
                stream.write_all(&len.to_le_bytes())?;

                // Write the actual message
                stream.write_all(&encoded)?;
                stream.flush()?;

                Ok(encoded)
                /*

                // Read response length
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf)?;
                let response_len = u32::from_le_bytes(len_buf);

                // Read response
                let mut response = vec![0u8; response_len as usize];
                stream.read_exact(&mut response)?;

                if let Ok(decoded) = bincode::deserialize::<EngineCommand>(&response) {
                    *command = decoded;
                } */
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }

    fn initialize(&self) {
        Logger::get_instance().log(LogLevel::Info, "Spawning squalr-cli privileged shell...", None);

        let ipc_server = self.ipc_server.clone();
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            match Self::spawn_squalr_cli_as_root() {
                Ok(child) => {
                    Logger::get_instance().log(LogLevel::Info, "Spawned squalr-cli as root.", None);
                    if let Ok(mut server) = ipc_server.write() {
                        *server = Some(child);

                        // Establish IPC connection
                        thread::sleep(std::time::Duration::from_millis(100)); // Give child process time to start // TODO: This is stupid just connect/retry
                        let name = IPC_SOCKET_PATH.to_fs_name::<NamedPipe>().unwrap();
                        match LocalSocketStream::connect(name) {
                            Ok(stream) => {
                                if let Ok(mut conn) = ipc_connection.write() {
                                    *conn = Some(stream);
                                }
                            }
                            Err(err) => {
                                Logger::get_instance().log(LogLevel::Error, &format!("Failed to establish IPC connection: {}", err), None);
                            }
                        }
                    }
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to spawn squalr-cli as root: {}", err), None);
                }
            }
        });
    }

    #[cfg(target_os = "android")]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("su")
            .arg("-c")
            .arg("squalr-cli --ipc-mode")
            .spawn()
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
