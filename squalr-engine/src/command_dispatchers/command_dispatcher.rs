use crate::command_dispatchers::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::command_dispatchers::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::engine_mode::EngineMode;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, Once};
use uuid::Uuid;

static mut INSTANCE: Option<CommandDispatcher> = None;
static INIT: Once = Once::new();

/// Orchestrates commands and responses to and from the engine.
pub struct CommandDispatcher {
    /// Defines the mode in which the engine is running.
    /// - Standalone engine is self-handling.
    /// - Unprivileged host sends data via ipc.
    /// - Privileged shell returns data via ipc.
    engine_mode: EngineMode,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(EngineResponse) + Send + Sync>>>>,
}

impl CommandDispatcher {
    fn new(engine_mode: EngineMode) -> Self {
        if engine_mode == EngineMode::UnprivilegedHost {
            InterProcessUnprivilegedHost::get_instance().initialize();
        } else if engine_mode == EngineMode::PrivilegedShell {
            InterProcessPrivilegedShell::get_instance().initialize();
        }

        CommandDispatcher {
            engine_mode,
            request_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_instance(engine_mode: EngineMode) {
        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(CommandDispatcher::new(engine_mode));
            });
        }
    }

    fn get_instance() -> &'static CommandDispatcher {
        unsafe {
            // If create_instance() has never been called before, default to standalone.
            if !INIT.is_completed() {
                panic!("Attempted to use command dispatcher before it was initialized");
            }

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn dispatch_command<F>(
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        let engine_mode = Self::get_instance().engine_mode;

        if engine_mode == EngineMode::Standalone {
            // For a standalone engine (the common case), we just immediately execute the command with a callback.
            callback(command.execute());
        } else {
            // For an inter-process engine (ie for Android), we dispatch the command to the priviliged root shell.
            let request_id = Uuid::new_v4();
            if let Ok(mut request_handles) = Self::get_instance().request_handles.lock() {
                request_handles.insert(request_id, Box::new(callback));
                InterProcessUnprivilegedHost::get_instance().dispatch_command(command, request_id);
            }
        }
    }

    pub fn dispatch_response(
        response: EngineResponse,
        request_id: Uuid,
    ) {
        let engine_mode = Self::get_instance().engine_mode;

        if engine_mode == EngineMode::Standalone {
            // For a standalone engine (the common case), we just immediately handle the response.
            CommandDispatcher::handle_response(response, request_id);
        } else {
            // For an inter-process engine (ie for Android), we dispatch the response back to the unprivileged host for handling.
            InterProcessPrivilegedShell::get_instance().dispatch_response(response, request_id)
        }
    }

    pub fn handle_response(
        response: EngineResponse,
        request_id: Uuid,
    ) {
        if let Ok(mut request_handles) = Self::get_instance().request_handles.lock() {
            if let Some(callback) = request_handles.remove(&request_id) {
                callback(response);
            }
        }
    }
}
