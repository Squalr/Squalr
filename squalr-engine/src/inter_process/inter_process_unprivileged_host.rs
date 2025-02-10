use crate::commands::engine_command::EngineCommand;
use crate::inter_process::inter_process_command_pipe::InterProcessCommandPipe;
use crate::inter_process::inter_process_connection::InterProcessConnection;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress::Event;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress::Response;
use crate::inter_process::inter_process_data_ingress::InterProcessDataIngress;
use crate::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::Once;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessUnprivilegedHost {
    privileged_shell_process: Arc<RwLock<Option<Child>>>,
    ipc_connection_ingress: Arc<RwLock<InterProcessConnection>>,
    ipc_connection_egress: Arc<RwLock<InterProcessConnection>>,
}

impl InterProcessUnprivilegedHost {
    pub fn get_instance() -> &'static InterProcessUnprivilegedHost {
        static mut INSTANCE: Option<InterProcessUnprivilegedHost> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = InterProcessUnprivilegedHost::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn new() -> InterProcessUnprivilegedHost {
        let instance = InterProcessUnprivilegedHost {
            privileged_shell_process: Arc::new(RwLock::new(None)),
            ipc_connection_ingress: Arc::new(RwLock::new(InterProcessConnection::new())),
            ipc_connection_egress: Arc::new(RwLock::new(InterProcessConnection::new())),
        };

        instance
    }

    pub fn initialize(&self) {
        Logger::get_instance().log(LogLevel::Info, "Spawning squalr-cli privileged shell...", None);

        let privileged_shell_process = self.privileged_shell_process.clone();
        let ipc_connection_ingress = self.ipc_connection_ingress.clone();
        let ipc_connection_egress = self.ipc_connection_egress.clone();

        thread::spawn(move || {
            // Self::spawn_privileged_cli(privileged_shell_process);
            Self::bind_to_inter_process_pipe(ipc_connection_ingress, true);
            Self::bind_to_inter_process_pipe(ipc_connection_egress.clone(), false);
            Self::listen_for_shell_events(ipc_connection_egress);
        });
    }

    pub fn dispatch_command(
        &self,
        command: EngineCommand,
        uuid: Uuid,
    ) {
        let ingress = InterProcessDataIngress::Command(command);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_shell(&self.ipc_connection_ingress, ingress, uuid) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC command: {}", err), None);
        }
    }

    fn spawn_privileged_cli(privileged_shell_process: Arc<RwLock<Option<Child>>>) {
        match Self::spawn_squalr_cli_as_root() {
            Ok(child) => {
                Logger::get_instance().log(LogLevel::Info, "Spawned squalr-cli as root.", None);

                // Update the server handle
                if let Ok(mut server) = privileged_shell_process.write() {
                    *server = Some(child);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to spawn squalr-cli as root: {}", err), None);
            }
        }
    }

    fn bind_to_inter_process_pipe(
        ipc_connection: Arc<RwLock<InterProcessConnection>>,
        is_ingress: bool,
    ) {
        match InterProcessCommandPipe::bind_to_inter_process_pipe(is_ingress) {
            Ok(stream) => {
                if let Ok(mut connection) = ipc_connection.write() {
                    connection.set_socket_stream(stream);
                } else {
                    Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on IPC connection.", None);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Error creating IPC manager: {}", err), None);
            }
        }
    }

    fn listen_for_shell_events(ipc_connection_egress: Arc<RwLock<InterProcessConnection>>) {
        thread::spawn(move || {
            loop {
                match InterProcessCommandPipe::ipc_receive_from_shell(&ipc_connection_egress) {
                    Ok((data_egress, uuid)) => {
                        Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                        match data_egress {
                            Event(engine_event) => {
                                SqualrEngine::broadcast_engine_event(engine_event);
                            }
                            Response(engine_response) => {
                                SqualrEngine::handle_response(engine_response, uuid);
                            }
                        }
                    }
                    Err(err) => {
                        // If we get an error here that indicates the socket is closed, and the parent process is closed. Shutdown this worker/child process too.
                        Logger::get_instance().log(LogLevel::Error, &format!("Parent connection lost: {}. Shutting down.", err), None);
                        std::process::exit(1);
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    #[cfg(any(target_os = "android"))]
    fn spawn_squalr_cli_as_root() -> std::io::Result<std::process::Child> {
        Logger::get_instance().log(LogLevel::Info, "Spawning privileged worker...", None);

        let child = Command::new("su")
            .arg("-c")
            .arg("/data/data/rust.squalr_android/files/squalr-cli")
            .arg("--ipc-mode")
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
