use crate::commands::engine_command::EngineCommand;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::process::Child;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;

pub struct InterProcessCommandDispatcher {
    /// The (optional) spawned child process with elevated privileges.
    ipc_server: Arc<RwLock<Option<Child>>>,
}

impl InterProcessCommandDispatcher {
    pub fn new() -> InterProcessCommandDispatcher {
        let instance = InterProcessCommandDispatcher {
            ipc_server: Arc::new(RwLock::new(None)),
        };

        instance.initialize();

        return instance;
    }

    pub fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        let encoded: Vec<u8> = bincode::serialize(&command).unwrap();
    }

    fn initialize(&self) {
        Logger::get_instance().log(LogLevel::Info, &"Spawning squalr-cli privileged shell...", None);

        let ipc_server = self.ipc_server.clone();
        thread::spawn(move || match Self::spawn_squalr_cli_as_root() {
            Ok(child) => {
                Logger::get_instance().log(LogLevel::Info, &"Spawned squalr-cli as root.", None);
                if let Ok(mut ipc_server) = ipc_server.write() {
                    *ipc_server = Some(child);
                }
            }
            Err(err) => {
                Logger::get_instance().log(LogLevel::Info, &format!("Failed to spawn squalr-cli as root: {}", err), None);
            }
        });
    }

    #[cfg(target_os = "android")]
    fn spawn_squalr_cli_as_root() -> io::Result<Child> {
        Command::new("su")
            .arg("-c")
            .arg("squalr-cli --ipc-mode")
            .spawn()
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
