use crate::command_handlers::command_handler::CommandHandler;
use crate::inter_process::inter_process_command_pipe::InterProcessCommandPipe;
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

        instance.initialize();
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
                match InterProcessCommandPipe::ipc_receive(&ipc_connection) {
                    Ok(mut engine_command) => {
                        Logger::get_instance().log(LogLevel::Info, "Dispatching IPC command...", None);
                        CommandHandler::handle_command(&mut engine_command);
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
