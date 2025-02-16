use crate::command_dispatchers::inter_process::inter_process_pipe_bidirectional::InterProcessPipeBidirectional;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::{Arc, Once, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessPrivilegedShell {
    ipc_connection: Arc<RwLock<Option<InterProcessPipeBidirectional>>>,
}

impl InterProcessPrivilegedShell {
    pub fn get_instance() -> &'static InterProcessPrivilegedShell {
        static mut INSTANCE: Option<InterProcessPrivilegedShell> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = InterProcessPrivilegedShell::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn new() -> InterProcessPrivilegedShell {
        let instance = InterProcessPrivilegedShell {
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance
    }

    pub fn initialize(&self) {
        if let Ok(mut ipc_connection) = self.ipc_connection.write() {
            match InterProcessPipeBidirectional::create() {
                Ok(new_connection) => *ipc_connection = Some(new_connection),
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Error creating bidirectional interprocess connection: {}", err), None);
                }
            }
        } else {
            Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on bidirectional interprocess connection.", None);
        }

        self.listen_for_host_requests();
    }

    pub fn dispatch_response(
        &self,
        engine_response: EngineResponse,
        request_id: Uuid,
    ) {
        if let Ok(ipc_connection) = self.ipc_connection.read() {
            if let Some(ipc_connection) = ipc_connection.as_ref() {
                if let Err(err) = ipc_connection.send(engine_response, request_id) {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC response: {}", err), None);
                }
            }
        }
    }

    fn listen_for_host_requests(&self) {
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            loop {
                if let Ok(ipc_connection) = ipc_connection.read() {
                    if let Some(ipc_connection) = ipc_connection.as_ref() {
                        match ipc_connection.receive::<EngineCommand>() {
                            Ok((engine_command, request_id)) => {
                                Logger::get_instance().log(LogLevel::Info, "Dispatching IPC response...", None);
                                let engine_response = engine_command.execute();
                                InterProcessPrivilegedShell::get_instance().dispatch_response(engine_response, request_id);
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
}
