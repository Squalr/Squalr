use crate::logging::cli_log_listener::CliLogListener;
use crate::runtime::cli::cli_runtime_mode::CliRuntimeMode;
use crate::runtime::ipc::ipc_runtime_mode::IpcRuntimeMode;
use crate::runtime::runtime_mode::RuntimeMode;
use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;

pub struct Runtime {
    mode: Box<dyn RuntimeMode>,
}

impl Runtime {
    pub fn new(args: Vec<String>) -> Self {
        let mode: Box<dyn RuntimeMode> = if args.len() > 1 && args[1] == "--ipc" {
            Box::new(IpcRuntimeMode::new())
        } else {
            Box::new(CliRuntimeMode::new())
        };

        Self { mode }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Hook into engine logging for the cli to display.
        let cli_log_listener = CliLogListener::new();
        Logger::get_instance().subscribe(cli_log_listener);

        // Initialize session manager.
        if let Ok(session_manager) = SessionManager::get_instance().read() {
            session_manager.initialize();
        } else {
            Logger::get_instance().log(LogLevel::Error, "Fatal error initializing session manager.", None);
            return Ok(());
        }

        self.mode.run()
    }

    pub fn shutdown(&mut self) {
        self.mode.shutdown()
    }
}
