use crate::commands::command_handlers::command_handler::CommandHandler;
use crate::commands::inter_process_command_pipe::InterProcessCommandPipe;
use interprocess::local_socket::prelude::LocalSocketStream;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

pub struct InterProcessCommandHandler {
    ipc_connection: Arc<RwLock<Option<LocalSocketStream>>>,
}

impl InterProcessCommandHandler {
    pub fn new() -> InterProcessCommandHandler {
        let instance = InterProcessCommandHandler {
            ipc_connection: Arc::new(RwLock::new(None)),
        };

        instance.initialize();
        instance
    }

    pub fn initialize(&self) {
        match InterProcessCommandPipe::create_connection() {
            Ok(stream) => {
                if let Ok(mut ipc_connection) = self.ipc_connection.write() {
                    *ipc_connection = Some(stream);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", err), None);
            }
        }

        let ipc_connection = self.ipc_connection.clone();

        thread::spawn(move || {
            loop {
                if let Ok(mut engine_command) = InterProcessCommandPipe::ipc_listen_command(&ipc_connection) {
                    CommandHandler::handle_command(&mut engine_command)
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
    }
}
