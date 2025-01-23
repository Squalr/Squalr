use std::io::Read;
use std::io::Write;

use crate::commands::command_handlers::memory;
use crate::commands::command_handlers::process;
use crate::commands::command_handlers::project;
use crate::commands::command_handlers::results;
use crate::commands::command_handlers::scan;
use crate::commands::command_handlers::settings;
use crate::commands::engine_command::EngineCommand;
use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::ToFsName;
use interprocess::local_socket::prelude::LocalSocketListener;
use interprocess::local_socket::traits::ListenerExt;
use interprocess::os::windows::local_socket::NamedPipe;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

const IPC_SOCKET_PATH: &str = if cfg!(windows) { "\\\\.\\pipe\\squalr-ipc" } else { "/tmp/squalr-ipc.sock" };

pub struct InterProcessCommandHandler {
    listener: Option<LocalSocketListener>,
}

impl InterProcessCommandHandler {
    pub fn new() -> InterProcessCommandHandler {
        let mut instance = InterProcessCommandHandler { listener: None };

        instance.initialize();
        instance
    }

    fn initialize(&mut self) {
        // Remove existing socket file if it exists
        if !cfg!(windows) {
            let _ = std::fs::remove_file(IPC_SOCKET_PATH);
        }

        // Create listener using ListenerOptions
        let name = IPC_SOCKET_PATH.to_fs_name::<NamedPipe>().unwrap();
        match ListenerOptions::new().name(name).create_sync() {
            Ok(listener) => {
                self.listener = Some(listener);
                self.start_ipc_listener();
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to create IPC listener: {}", err), None);
            }
        }
    }

    pub fn handle_command(
        &self,
        command: &mut EngineCommand,
    ) {
        match command {
            EngineCommand::Memory(cmd) => memory::handle_memory_command(cmd),
            EngineCommand::Process(cmd) => process::handle_process_command(cmd),
            EngineCommand::Project(cmd) => project::handle_project_command(cmd),
            EngineCommand::Results(cmd) => results::handle_results_command(cmd),
            EngineCommand::Scan(cmd) => scan::handle_scan_command(cmd),
            EngineCommand::Settings(cmd) => settings::handle_settings_command(cmd),
        }
    }

    fn start_ipc_listener(&self) {
        if let Some(listener) = &self.listener {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        loop {
                            // Read message length
                            let mut len_buf = [0u8; 4];
                            if stream.read_exact(&mut len_buf).is_err() {
                                break;
                            }
                            let msg_len = u32::from_le_bytes(len_buf);

                            // Read message
                            let mut msg = vec![0u8; msg_len as usize];
                            if stream.read_exact(&mut msg).is_err() {
                                break;
                            }

                            // Deserialize and handle command
                            if let Ok(mut command) = bincode::deserialize::<EngineCommand>(&msg) {
                                self.handle_command(&mut command);

                                // Send response
                                if let Ok(response) = bincode::serialize(&command) {
                                    let len = response.len() as u32;
                                    if stream.write_all(&len.to_le_bytes()).is_ok() {
                                        let _ = stream.write_all(&response);
                                        let _ = stream.flush();
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        Logger::get_instance().log(LogLevel::Error, &format!("Failed to accept IPC connection: {}", err), None);
                    }
                }
            }
        }
    }
}
