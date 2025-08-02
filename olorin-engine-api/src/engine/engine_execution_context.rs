use crate::commands::{engine_command::EngineCommand, engine_command_response::EngineCommandResponse};
use crate::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use crate::events::engine_event::EngineEvent;
use crate::events::engine_event::EngineEventRequest;
use crate::events::process::process_event::ProcessEvent;
use crate::events::project::project_event::ProjectEvent;
use crate::events::project_items::project_items_event::ProjectItemsEvent;
use crate::events::scan_results::scan_results_event::ScanResultsEvent;
use crate::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use olorin_engine_common::logging::file_system_logger::FileSystemLogger;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Exposes the ability to send commands to the engine, and handle events from the engine.
pub struct EngineExecutionContext {
    /// The bindings that allow sending commands to the engine.
    engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>>,

    /// All event listeners that are listening for particular engine events.
    event_listeners: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,

    // Routes logs to the file system, as well as any optional subscribers to log events, such as output in the GUI.
    file_system_logger: Arc<FileSystemLogger>,
}

impl EngineExecutionContext {
    pub fn new(engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>>) -> Arc<Self> {
        let execution_context = Arc::new(EngineExecutionContext {
            engine_bindings,
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
            file_system_logger: Arc::new(FileSystemLogger::new()),
        });

        execution_context
    }

    pub fn initialize(&self) {
        self.start_event_dispatcher();

        // Start the log event sending now that the engine is initialized. This will send all backlogged messages to listeners.
        self.get_logger().start_log_event_sender();
    }

    pub fn get_bindings(&self) -> &Arc<RwLock<dyn EngineApiUnprivilegedBindings>> {
        &self.engine_bindings
    }

    /// Gets the file system logger that routes log events to the log file.
    pub fn get_logger(&self) -> &Arc<FileSystemLogger> {
        &self.file_system_logger
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
            Err(error) => {
                log::error!("Error listening for engine event: {}", error);
            }
        }
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
                if let Err(error) = engine_bindings.dispatch_command(engine_command, Box::new(callback)) {
                    log::error!("Error dispatching engine command: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire unprivileged engine bindings read lock for commands: {}", error);
            }
        }
    }
    /// Starts listening for all engine events, and routes specific events to any listeners for that event type.
    fn start_event_dispatcher(&self) {
        let event_receiver = match self.engine_bindings.read() {
            Ok(bindings) => match bindings.subscribe_to_engine_events() {
                Ok(receiver) => receiver,
                Err(error) => {
                    log::error!("Failed to subscribe to engine events: {}", error);
                    return;
                }
            },
            Err(error) => {
                log::error!("Failed to acquire engine bindings read lock: {}", error);
                return;
            }
        };
        let event_listeners = self.event_listeners.clone();

        std::thread::spawn(move || {
            loop {
                match event_receiver.recv() {
                    Ok(engine_event) => Self::route_engine_event(&event_listeners, engine_event),
                    Err(error) => {
                        log::error!("Fatal error listening for engine events: {}", error);
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
            EngineEvent::ProjectItems(project_items_event) => match project_items_event {
                ProjectItemsEvent::ProjectItemsChanged { project_items_changed_event } => {
                    Self::dispatch_engine_event(&event_listeners, project_items_changed_event);
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
            Err(error) => {
                log::error!("Error dispatching engine event: {}", error);
            }
        }
    }
}
