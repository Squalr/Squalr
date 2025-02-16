use crate::command_dispatchers::command_dispatcher::CommandDispatcher;
use crate::command_dispatchers::inter_process::inter_process_pipe_bidirectional::InterProcessPipeBidirectional;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
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
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
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
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance
    }

    pub fn initialize(&self) {
        Logger::get_instance().log(LogLevel::Info, "Spawning squalr-cli privileged shell...", None);

        let privileged_shell_process = self.privileged_shell_process.clone();
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            Self::spawn_privileged_cli(privileged_shell_process);
            Self::bind_to_inter_process_pipe(ipc_connection.clone());
            Self::listen_for_shell_events(ipc_connection);
        });
    }

    pub fn dispatch_command(
        &self,
        command: EngineCommand,
        request_id: Uuid,
    ) {
        if let Ok(ipc_connection) = self.ipc_connection.read() {
            if let Some(ipc_connection) = ipc_connection.as_ref() {
                if let Err(err) = ipc_connection.send(command, request_id) {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC command: {}", err), None);
                }
            }
        }
    }

    fn listen_for_shell_events(ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>) {
        thread::spawn(move || {
            loop {
                if let Ok(ipc_connection) = ipc_connection.read() {
                    if let Some(ipc_connection) = ipc_connection.as_ref() {
                        match ipc_connection.receive::<EngineResponse>() {
                            Ok((engine_response, request_id)) => {
                                Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                                CommandDispatcher::handle_response(engine_response, request_id);
                            }
                            Err(err) => {
                                Logger::get_instance().log(LogLevel::Error, &format!("Parent connection lost: {}. Shutting down.", err), None);
                                std::process::exit(1);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
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

    fn bind_to_inter_process_pipe(ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>) {
        if let Ok(mut ipc_connection) = ipc_connection.write() {
            match InterProcessPipeBidirectional::bind() {
                Ok(bound_connection) => *ipc_connection = Some(bound_connection),
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Error creating bidirectional interprocess connection: {}", err), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on bidirectional interprocess connection.", None);
        }
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
