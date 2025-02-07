use crate::command_handlers::command_handler::CommandHandler;
use crate::events::engine_event::EngineEvent;
use crate::inter_process::inter_process_command_pipe::InterProcessCommandPipe;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress;
use crate::inter_process::inter_process_data_ingress::InterProcessDataIngress::Command;
use crate::responses::engine_response::EngineResponse;
use interprocess::local_socket::prelude::LocalSocketStream;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;
use std::sync::Once;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

pub struct InterProcessPrivilegedShell {
    ipc_connection: Arc<RwLock<Option<LocalSocketStream>>>,
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
        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            match InterProcessCommandPipe::create_inter_process_pipe() {
                Ok(stream) => {
                    if let Ok(mut ipc_connection) = ipc_connection.write() {
                        *ipc_connection = Some(stream);
                    }
                }
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("{}", err), None);
                }
            }

            loop {
                match InterProcessCommandPipe::ipc_receive_from_host(&ipc_connection) {
                    Ok(data_ingress) => {
                        Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                        match data_ingress {
                            Command(engine_command) => {
                                CommandHandler::handle_command(engine_command);
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

    pub fn dispatch_event(
        &self,
        event: EngineEvent,
    ) {
        let egress = InterProcessDataEgress::Event(event);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_host(&self.ipc_connection, egress) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC event: {}", err), None);
        }
    }

    pub fn dispatch_response(
        &self,
        response: EngineResponse,
    ) {
        let egress = InterProcessDataEgress::Response(response);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_host(&self.ipc_connection, egress) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC response: {}", err), None);
        }
    }
}
