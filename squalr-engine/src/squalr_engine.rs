use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use squalr_engine_architecture::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::{Arc, Once};
use uuid::Uuid;

static mut INSTANCE: Option<SqualrEngine> = None;
static INIT: Once = Once::new();

/// Defines the mode of operation for Squalr's engine.
#[derive(Clone, Copy, PartialEq)]
pub enum EngineMode {
    /// Standalone mode grants full functionality. This is the most common mode.
    Standalone,

    /// In Unprivileged Host mode, we only send and receive engine commands from the privileged shell.
    /// This is necessary on some platforms like Android, where the main process may be unprivileged.
    UnprivilegedHost,

    /// The privileged shell does heavy lifting (scanning, debugging, etc) and sends responses to the host.
    PrivilegedShell,
}

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    /// Defines the mode in which the engine is running.
    /// - Standalone engine is self-handling.
    /// - Unprivileged host sends data via ipc.
    /// - Privileged shell returns data via ipc.
    engine_mode: EngineMode,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn FnOnce(EngineResponse) + Send + Sync>>>>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        if engine_mode == EngineMode::UnprivilegedHost {
            InterProcessUnprivilegedHost::get_instance().initialize();
        } else if engine_mode == EngineMode::PrivilegedShell {
            InterProcessPrivilegedShell::get_instance().initialize();
        }

        SqualrEngine {
            engine_mode,
            request_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn create_instance(engine_mode: EngineMode) {
        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(SqualrEngine::new(engine_mode));
            });
        }
    }

    fn get_instance() -> &'static SqualrEngine {
        unsafe {
            // If create_instance() has never been called before, default to standalone.
            if !INIT.is_completed() {
                panic!("Attempted to use engine before it was initialized");
            }

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn initialize(engine_mode: EngineMode) {
        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        Self::create_instance(engine_mode);

        match engine_mode {
            EngineMode::Standalone | EngineMode::PrivilegedShell => {
                if let Err(err) = ProcessQuery::start_monitoring() {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
                }
            }
            EngineMode::UnprivilegedHost => {}
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

    pub fn dispatch_command_async<F>(
        command: EngineCommand,
        callback: F,
    ) where
        F: Fn(EngineResponse) + Send + Sync + 'static,
    {
        std::thread::spawn(move || {
            Self::dispatch_command(command, callback);
        });
    }

    pub fn dispatch_response(
        response: EngineResponse,
        request_id: Uuid,
    ) {
        let engine_mode = Self::get_instance().engine_mode;

        if engine_mode == EngineMode::Standalone {
            // For a standalone engine (the common case), we just immediately handle the response.
            SqualrEngine::handle_response(response, request_id);
        } else {
            // For an inter-process engine (ie for Android), we dispatch the response back to the unprivileged host for handling.
            InterProcessPrivilegedShell::get_instance().dispatch_response(response, request_id)
        }
    }

    pub fn dispatch_response_async(
        response: EngineResponse,
        request_id: Uuid,
    ) {
        std::thread::spawn(move || {
            Self::dispatch_response(response, request_id);
        });
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
