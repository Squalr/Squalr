use crate::engine_bindings::engine_unprivileged_bindings::EngineUnprivilegedBindings;
use crate::engine_bindings::interprocess::interprocess_unprivileged_host::InterprocessUnprivilegedHost;
use crate::engine_bindings::standalone::standalone_unprivileged_interface::StandaloneUnprivilegedInterface;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command_response::EngineCommandResponse;
use squalr_engine_api::events::engine_event::EngineEventRequest;
use squalr_engine_api::events::process::process_event::ProcessEvent;
use squalr_engine_api::events::project::project_event::ProjectEvent;
use squalr_engine_api::events::scan_results::scan_results_event::ScanResultsEvent;
use squalr_engine_api::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use squalr_engine_api::{commands::engine_command::EngineCommand, events::engine_event::EngineEvent};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Exposes the ability to send commands to the engine, and handle events from the engine.
pub struct EngineExecutionContext {
    /// The bindings that allow sending commands to the engine.
    engine_bindings: Arc<RwLock<dyn EngineUnprivilegedBindings>>,

    /// All event listeners that are listening for particular engine events.
    event_listeners: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,
}

impl EngineExecutionContext {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let engine_bindings: Arc<RwLock<dyn EngineUnprivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandaloneUnprivilegedInterface::new())),
            EngineMode::PrivilegedShell => unreachable!("Unprivileged execution context should never be created from a privileged shell."),
            EngineMode::UnprivilegedHost => Arc::new(RwLock::new(InterprocessUnprivilegedHost::new())),
        };

        let execution_context = Arc::new(EngineExecutionContext {
            engine_bindings,
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
        });

        execution_context
    }

    pub fn initialize(
        &self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
    ) {
        match self.engine_bindings.write() {
            Ok(mut engine_bindings) => {
                if let Err(err) = engine_bindings.initialize(engine_privileged_state) {
                    log::error!("Error initializing unprivileged engine bindings: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire unprivileged engine bindings write lock: {}", err);
            }
        }

        self.start_event_dispatcher();
    }

    /// Dispatches a command to the engine. Direct usage is generally not advised unless you know what you are doing.
    /// Instead, create `{Command}Request` instances and call `.send()` directly on them.
    /// This is only made public to support direct usage by CLIs and other features that may need direct access.
    pub fn dispatch_command<F>(
        self: &Arc<Self>,
        engine_command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineCommandResponse) + Send + Sync + 'static,
    {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(err) = engine_bindings.dispatch_command(engine_command, Box::new(callback)) {
                    log::error!("Error dispatching engine command: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire unprivileged engine bindings read lock for commands: {}", err);
            }
        }
    }

    /// Registers a listener for each time a particular engine event is fired.
    pub fn listen_for_engine_event<E: EngineEventRequest + 'static>(
        &self,
        callback: impl Fn(&E) + Send + Sync + 'static,
    ) {
        match self.event_listeners.write() {
            Ok(mut event_listeners) => {
                let callbacks = event_listeners
                    .entry(TypeId::of::<E>())
                    .or_insert_with(Vec::new);
                callbacks.push(Box::new(move |event| {
                    if let Some(event) = event.downcast_ref::<E>() {
                        callback(event);
                    }
                }));
            }
            Err(err) => {
                log::error!("Error listening for engine event: {}", err);
            }
        }
    }

    /// Starts listening for all engine events, and routes specific events to any listeners for that event type.
    fn start_event_dispatcher(&self) {
        let event_receiver = match self.engine_bindings.read() {
            Ok(bindings) => match bindings.subscribe_to_engine_events() {
                Ok(receiver) => receiver,
                Err(err) => {
                    log::error!("Failed to subscribe to engine events: {}", err);
                    return;
                }
            },
            Err(err) => {
                log::error!("Failed to acquire engine bindings read lock: {}", err);
                return;
            }
        };
        let event_listeners = self.event_listeners.clone();

        std::thread::spawn(move || {
            loop {
                match event_receiver.recv() {
                    Ok(engine_event) => Self::route_engine_event(&event_listeners, engine_event),
                    Err(err) => {
                        log::error!("Fatal error listening for engine events: {}", err);
                        return;
                    }
                }
            }
        });
    }

    /// Deconstructs an engine event to extract the particular event structure being sent, and routes it to the proper event listeners.
    fn route_engine_event(
        event_listeners: &Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,
        engine_event: EngineEvent,
    ) {
        match engine_event {
            EngineEvent::Process(process_event) => match process_event {
                ProcessEvent::ProcessChanged { process_changed_event } => {
                    Self::dispatch_engine_event(&event_listeners, process_changed_event);
                }
            },
            EngineEvent::Project(project_event) => match project_event {
                ProjectEvent::ProjectClosed { project_closed_event } => {
                    Self::dispatch_engine_event(&event_listeners, project_closed_event);
                }
                ProjectEvent::ProjectCreated { project_created_event } => {
                    Self::dispatch_engine_event(&event_listeners, project_created_event);
                }
                ProjectEvent::ProjectDeleted { project_deleted_event } => {
                    Self::dispatch_engine_event(&event_listeners, project_deleted_event);
                }
            },
            EngineEvent::ScanResults(process_event) => match process_event {
                ScanResultsEvent::ScanResultsUpdated { scan_results_updated_event } => {
                    Self::dispatch_engine_event(&event_listeners, scan_results_updated_event);
                }
            },
            EngineEvent::TrackableTask(trackable_task_event) => match trackable_task_event {
                TrackableTaskEvent::ProgressChanged { progress_changed_event } => {
                    Self::dispatch_engine_event(&event_listeners, progress_changed_event);
                }
            },
        }
    }

    /// Dispatches a particular engine event to all listeners for its event type.
    fn dispatch_engine_event<E: 'static + Any + Send + Sync>(
        event_listeners: &Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,
        event: E,
    ) {
        match event_listeners.read() {
            Ok(event_listeners) => {
                if let Some(callbacks) = event_listeners.get(&TypeId::of::<E>()) {
                    for callback in callbacks {
                        callback(&event);
                    }
                }
            }
            Err(err) => {
                log::error!("Error dispatching engine event: {}", err);
            }
        }
    }
}
