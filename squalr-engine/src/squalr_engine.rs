use crate::commands::command_dispatcher::CommandDispatcher;
use crate::commands::engine_command::EngineCommand;
use crate::events::engine_event::EngineEvent;
use crate::events::event_dispatcher::EventDispatcher;
use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use crate::responses::engine_response::EngineResponse;
use crate::responses::response_dispatcher::ResponseDispatcher;
use squalr_engine_architecture::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::collections::HashMap;
use std::sync::mpsc::SendError;
use std::sync::{Arc, Once};
use std::sync::{Mutex, mpmc};
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
    /// Handles sending commands to the engine.
    command_dispatcher: Arc<Mutex<CommandDispatcher>>,

    /// Handles sending events from the engine to the GUI/CLI/etc.
    event_dispatcher: Arc<Mutex<EventDispatcher>>,

    /// Handles sending responses from the engine to the GUI/CLI/etc.
    response_dispatcher: Arc<Mutex<ResponseDispatcher>>,

    /// Handles broadcasting events from the engine.
    event_sender: mpmc::Sender<EngineEvent>,

    /// Clonable receiver for receiving events from the engine.
    event_receiver: mpmc::Receiver<EngineEvent>,

    /// A map of outgoing requests that are awaiting an engine response.
    request_handles: Arc<Mutex<HashMap<Uuid, Box<dyn Fn(EngineResponse) + Send + Sync>>>>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        // Standalone engine is self-handling.
        // Unprivileged host sends data via ipc.
        // Privileged shell returns data via ipc.
        let ingress_dispatcher_type = match engine_mode {
            EngineMode::Standalone => DispatcherType::Standalone,
            EngineMode::UnprivilegedHost => DispatcherType::InterProcess,
            EngineMode::PrivilegedShell => DispatcherType::None,
        };
        let egress_dispatcher_type = match engine_mode {
            EngineMode::Standalone => DispatcherType::Standalone,
            EngineMode::UnprivilegedHost => DispatcherType::None,
            EngineMode::PrivilegedShell => DispatcherType::InterProcess,
        };

        if engine_mode == EngineMode::UnprivilegedHost {
            InterProcessUnprivilegedHost::get_instance().initialize();
        } else if engine_mode == EngineMode::PrivilegedShell {
            InterProcessPrivilegedShell::get_instance().initialize();
        }

        let (event_sender, event_receiver) = mpmc::channel();

        SqualrEngine {
            command_dispatcher: Arc::new(Mutex::new(CommandDispatcher::new(ingress_dispatcher_type))),
            event_dispatcher: Arc::new(Mutex::new(EventDispatcher::new(egress_dispatcher_type))),
            response_dispatcher: Arc::new(Mutex::new(ResponseDispatcher::new(egress_dispatcher_type))),
            event_sender: event_sender,
            event_receiver: event_receiver,
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
            Self::create_instance(EngineMode::Standalone);

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn initialize(engine_mode: EngineMode) {
        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        Self::create_instance(engine_mode);

        if let Err(err) = ProcessQuery::start_monitoring() {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
        }
    }

    pub fn dispatch_command<F>(
        command: EngineCommand,
        callback: F,
    ) where
        F: Fn(EngineResponse) + Send + Sync + 'static,
    {
        if let Ok(dispatcher) = Self::get_instance().command_dispatcher.lock() {
            let command_to_dispatch = dispatcher.prepare_dispatch(command);

            if let Ok(mut request_handles) = Self::get_instance().request_handles.lock() {
                request_handles.insert(command_to_dispatch.get_id(), Box::new(callback));
            }

            command_to_dispatch.execute();
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

    pub fn dispatch_event(event: EngineEvent) {
        if let Ok(dispatcher) = Self::get_instance().event_dispatcher.lock() {
            dispatcher.dispatch_event(event, Uuid::new_v4());
        }
    }

    pub fn dispatch_event_async(event: EngineEvent) {
        std::thread::spawn(move || {
            Self::dispatch_event(event);
        });
    }

    pub fn get_engine_event_receiver() -> mpmc::Receiver<EngineEvent> {
        SqualrEngine::get_instance().event_receiver.clone()
    }

    pub fn broadcast_engine_event(event: EngineEvent) {
        if let Err(err) = SqualrEngine::get_instance().event_sender.send(event) {
            Logger::get_instance().log(LogLevel::Error, &format!("Failed to broadcast event: {}", err), None);
        }
    }

    pub fn dispatch_response(
        response: EngineResponse,
        uuid: Uuid,
    ) {
        if let Ok(dispatcher) = Self::get_instance().response_dispatcher.lock() {
            dispatcher.dispatch_response(response, uuid);
        }
    }

    pub fn dispatch_response_async(
        response: EngineResponse,
        uuid: Uuid,
    ) {
        std::thread::spawn(move || {
            Self::dispatch_response(response, uuid);
        });
    }

    pub fn handle_response(
        response: EngineResponse,
        uuid: Uuid,
    ) {
        if let Ok(mut request_handles) = Self::get_instance().request_handles.lock() {
            if let Some(callback) = request_handles.get(&uuid) {
                callback(response);
            }

            request_handles.remove(&uuid);
        }
    }
}
