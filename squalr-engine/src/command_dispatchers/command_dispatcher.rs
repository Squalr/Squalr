use crate::command_dispatchers::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::command_dispatchers::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::engine_mode::EngineMode;
use crate::squalr_engine::SqualrEngine;

/// Orchestrates commands and responses to and from the engine.
pub struct CommandDispatcher {}

impl CommandDispatcher {
    pub fn initialize(engine_mode: EngineMode) {
        if engine_mode == EngineMode::UnprivilegedHost {
            InterProcessUnprivilegedHost::get_instance().initialize();
        } else if engine_mode == EngineMode::PrivilegedShell {
            InterProcessPrivilegedShell::get_instance().initialize();
        }
    }

    pub fn dispatch_command<F>(
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        let engine_mode = SqualrEngine::get_engine_mode();

        if engine_mode == EngineMode::Standalone {
            // For a standalone engine (the common case), we just immediately execute the command with a callback.
            callback(command.execute());
        } else {
            InterProcessUnprivilegedHost::get_instance().dispatch_command(command, callback);
        }
    }
}
