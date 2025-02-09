use crate::commands::command_handler::CommandHandler;
use crate::events::engine_event::EngineEvent;
use crate::inter_process::inter_process_command_pipe::InterProcessCommandPipe;
use crate::inter_process::inter_process_connection::InterProcessConnection;
use crate::inter_process::inter_process_data_egress::InterProcessDataEgress;
use crate::inter_process::inter_process_data_ingress::InterProcessDataIngress::Command;
use crate::responses::engine_response::EngineResponse;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::{Arc, Once, RwLock};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct InterProcessPrivilegedShell {
    ipc_connection_ingress: Arc<RwLock<InterProcessConnection>>,
    ipc_connection_egress: Arc<RwLock<InterProcessConnection>>,
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
            ipc_connection_ingress: Arc::new(RwLock::new(InterProcessConnection::new())),
            ipc_connection_egress: Arc::new(RwLock::new(InterProcessConnection::new())),
        };

        instance
    }

    pub fn initialize(&self) {
        match InterProcessCommandPipe::create_inter_process_pipe(true) {
            Ok(stream) => {
                if let Ok(mut connection) = self.ipc_connection_ingress.write() {
                    connection.set_socket_stream(stream);
                } else {
                    Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on IPC connection.", None);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", err), None);
            }
        }

        match InterProcessCommandPipe::create_inter_process_pipe(false) {
            Ok(stream) => {
                if let Ok(mut connection) = self.ipc_connection_egress.write() {
                    connection.set_socket_stream(stream);
                } else {
                    Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on IPC connection.", None);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", err), None);
            }
        }

        self.listen_for_host_events();
    }

    pub fn dispatch_event(
        &self,
        event: EngineEvent,
        uuid: Uuid,
    ) {
        let egress = InterProcessDataEgress::Event(event);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_host(&self.ipc_connection_egress, egress, uuid) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC event: {}", err), None);
        }
    }

    pub fn dispatch_response(
        &self,
        response: EngineResponse,
        uuid: Uuid,
    ) {
        let egress = InterProcessDataEgress::Response(response);

        if let Err(err) = InterProcessCommandPipe::ipc_send_to_host(&self.ipc_connection_egress, egress, uuid) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to send IPC response: {}", err), None);
        }
    }

    fn listen_for_host_events(&self) {
        let ipc_connection_ingress = self.ipc_connection_ingress.clone();

        thread::spawn(move || {
            loop {
                match InterProcessCommandPipe::ipc_receive_from_host(&ipc_connection_ingress) {
                    Ok((data_ingress, uuid)) => {
                        Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                        match data_ingress {
                            Command(engine_command) => {
                                engine_command.handle(uuid);
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
}
