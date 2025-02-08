use crate::commands::engine_command::EngineCommand;
use crate::event_handlers::event_handler::EventHandler;
use crate::inter_process::inter_process_command_pipe::InterProcessCommandPipe;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress::Event;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress::Response;
use crate::inter_process::inter_process_data_ingress::InterProcessDataIngress;
use crate::response_handlers::response_handler::ResponseHandler;
use interprocess::local_socket::prelude::LocalSocketStream;
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
    ipc_server: Arc<RwLock<Option<Child>>>,
    ipc_connection: Arc<RwLock<Option<LocalSocketStream>>>,
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
            ipc_server: Arc::new(RwLock::new(None)),
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance
    }

    pub fn initialize(&self) {
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

                    match InterProcessCommandPipe::bind_to_inter_process_pipe() {
                        Ok(stream) => {
                            if let Ok(mut ipc_connection_ref) = ipc_connection.write() {
                                *ipc_connection_ref = Some(stream);

                                Self::listen_for_shell_events(ipc_connection.clone());
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
        command: EngineCommand,
        uuid: Uuid,
    ) {
        let ingress = InterProcessDataIngress::Command(command);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_shell(&self.ipc_connection, ingress, uuid) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC command: {}", err), None);
        }
    }

    fn listen_for_shell_events(ipc_connection: Arc<RwLock<Option<LocalSocketStream>>>) {
        thread::spawn(move || {
            loop {
                match InterProcessCommandPipe::ipc_receive_from_shell(&ipc_connection) {
                    Ok((data_egress, uuid)) => {
                        Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                        match data_egress {
                            Event(engine_event) => {
                                EventHandler::handle_event(engine_event, uuid);
                            }
                            Response(engine_response) => {
                                ResponseHandler::handle_response(engine_response, uuid);
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
