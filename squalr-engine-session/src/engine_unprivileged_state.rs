use crate::logging::log_dispatcher::{LogDispatcher, LogDispatcherOptions};
use crate::registries::symbol_registry_mirror::SymbolRegistryMirror;
use crossbeam_channel::bounded;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::registry::get_snapshot::registry_get_snapshot_request::RegistryGetSnapshotRequest;
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::events::process::process_event::ProcessEvent;
use squalr_engine_api::events::project::project_event::ProjectEvent;
use squalr_engine_api::events::project_items::project_items_event::ProjectItemsEvent;
use squalr_engine_api::events::registry::registry_event::RegistryEvent;
use squalr_engine_api::events::scan_results::scan_results_event::ScanResultsEvent;
use squalr_engine_api::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue,
};
use squalr_engine_api::structures::projects::project_manager::ProjectManager;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

/// Exposes the ability to send commands to the engine and handle events from the engine.
pub struct EngineUnprivilegedState {
    /// The bindings that allow sending commands to the engine.
    engine_api_unprivileged_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>>,
    /// All event listeners that are listening for particular engine events.
    event_listeners: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,
    /// Routes logs to the file system as well as optional subscribers to log events.
    file_system_logger: Arc<LogDispatcher>,
    /// Project manager for organizing and manipulating projects.
    project_manager: Arc<ProjectManager>,
    /// Local mirror of the privileged symbol registry snapshot.
    symbol_registry_mirror: Arc<RwLock<SymbolRegistryMirror>>,
    /// Local compatibility registry kept in sync from snapshots for legacy metadata and formatting helpers.
    symbol_registry_compatibility: Arc<SymbolRegistry>,
}

#[derive(Clone, Copy)]
pub struct EngineUnprivilegedStateOptions {
    pub enable_console_logging: bool,
}

impl Default for EngineUnprivilegedStateOptions {
    fn default() -> Self {
        Self { enable_console_logging: true }
    }
}

impl EngineExecutionContext for EngineUnprivilegedState {
    fn get_bindings(&self) -> &Arc<RwLock<dyn EngineApiUnprivilegedBindings>> {
        &self.engine_api_unprivileged_bindings
    }

    fn get_project_manager(&self) -> &Arc<ProjectManager> {
        &self.project_manager
    }

    fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        EngineUnprivilegedState::get_default_anonymous_value_string_format(self, data_type_ref)
    }

    fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, SymbolRegistryError> {
        EngineUnprivilegedState::anonymize_value(self, data_value, anonymous_value_string_format)
    }

    fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue> {
        self.symbol_registry_compatibility
            .get_default_value(data_type_ref)
    }

    fn resolve_symbolic_struct_definition(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition> {
        self.symbol_registry_compatibility
            .get(symbolic_struct_ref_id)
            .as_deref()
            .cloned()
    }
}

impl EngineUnprivilegedState {
    pub fn new(engine_api_unprivileged_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>>) -> Arc<Self> {
        Self::new_with_options(engine_api_unprivileged_bindings, EngineUnprivilegedStateOptions::default())
    }

    pub fn new_with_options(
        engine_api_unprivileged_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>>,
        options: EngineUnprivilegedStateOptions,
    ) -> Arc<Self> {
        let project_manager = Arc::new(ProjectManager::new());

        Arc::new(EngineUnprivilegedState {
            engine_api_unprivileged_bindings,
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
            file_system_logger: Arc::new(LogDispatcher::new_with_options(LogDispatcherOptions {
                enable_console_output: options.enable_console_logging,
            })),
            project_manager,
            symbol_registry_mirror: Arc::new(RwLock::new(SymbolRegistryMirror::default())),
            symbol_registry_compatibility: Arc::new(SymbolRegistry::new()),
        })
    }

    pub fn initialize(self: &Arc<Self>) {
        self.start_event_dispatcher();
        self.refresh_symbol_registry_snapshot();
    }

    /// Gets the file system logger that routes log events to the log file.
    pub fn get_logger(&self) -> &Arc<LogDispatcher> {
        &self.file_system_logger
    }

    pub fn get_symbol_registry_mirror(&self) -> &Arc<RwLock<SymbolRegistryMirror>> {
        &self.symbol_registry_mirror
    }

    pub fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.symbol_registry_compatibility
            .get_registered_data_type_refs()
    }

    pub fn is_registered_data_type_ref(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.symbol_registry_compatibility.is_valid(data_type_ref)
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.symbol_registry_compatibility
            .get_default_anonymous_value_string_format(data_type_ref)
    }

    pub fn get_supported_anonymous_value_string_formats(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<AnonymousValueStringFormat> {
        self.symbol_registry_compatibility
            .get_supported_anonymous_value_string_formats(data_type_ref)
    }

    pub fn validate_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.symbol_registry_compatibility
            .validate_value_string(data_type_ref, anonymous_value_string)
    }

    pub fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, SymbolRegistryError> {
        self.symbol_registry_compatibility
            .deanonymize_value_string(data_type_ref, anonymous_value_string)
    }

    pub fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, SymbolRegistryError> {
        self.symbol_registry_compatibility
            .anonymize_value(data_value, anonymous_value_string_format)
    }

    pub fn anonymize_value_to_supported_formats(
        &self,
        data_value: &DataValue,
    ) -> Result<Vec<AnonymousValueString>, SymbolRegistryError> {
        self.symbol_registry_compatibility
            .anonymize_value_to_supported_formats(data_value)
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

    /// Dispatches a command to the engine.
    pub fn dispatch_command<F>(
        self: &Arc<Self>,
        privileged_command: PrivilegedCommand,
        callback: F,
    ) where
        F: FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static,
    {
        match self.engine_api_unprivileged_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(error) = engine_bindings.dispatch_privileged_command(privileged_command, Box::new(callback)) {
                    log::error!("Error dispatching engine command: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire unprivileged engine bindings read lock for commands: {}", error);
            }
        }
    }

    /// Starts listening for all engine events and routes specific events to listeners for that event type.
    fn start_event_dispatcher(self: &Arc<Self>) {
        let event_receiver = match self.engine_api_unprivileged_bindings.read() {
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
        let engine_unprivileged_state = self.clone();

        std::thread::spawn(move || {
            loop {
                match event_receiver.recv() {
                    Ok(engine_event_envelope) => Self::route_engine_event(&engine_unprivileged_state, engine_event_envelope),
                    Err(error) => {
                        log::error!("Fatal error listening for engine events: {}", error);
                        return;
                    }
                }
            }
        });
    }

    /// Deconstructs an engine event to extract the particular event structure being sent and routes it to listeners.
    fn route_engine_event(
        engine_unprivileged_state: &Arc<Self>,
        engine_event_envelope: EngineEventEnvelope,
    ) {
        if !engine_unprivileged_state.ensure_symbol_registry_snapshot_current(engine_event_envelope.get_registry_generation()) {
            log::error!(
                "Failed to refresh symbol registry mirror to generation {} before dispatching engine event.",
                engine_event_envelope.get_registry_generation()
            );
        }

        let engine_event = engine_event_envelope.into_engine_event();

        match engine_event {
            EngineEvent::Process(process_event) => match process_event {
                ProcessEvent::ProcessChanged { process_changed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, process_changed_event);
                }
            },
            EngineEvent::Project(project_event) => match project_event {
                ProjectEvent::ProjectClosed { project_closed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, project_closed_event);
                }
                ProjectEvent::ProjectCreated { project_created_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, project_created_event);
                }
                ProjectEvent::ProjectDeleted { project_deleted_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, project_deleted_event);
                }
            },
            EngineEvent::ProjectItems(project_items_event) => match project_items_event {
                ProjectItemsEvent::ProjectItemsChanged { project_items_changed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, project_items_changed_event);
                }
            },
            EngineEvent::Registry(registry_event) => match registry_event {
                RegistryEvent::Changed { registry_changed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, registry_changed_event);
                }
            },
            EngineEvent::ScanResults(scan_results_event) => match scan_results_event {
                ScanResultsEvent::ScanResultsUpdated { scan_results_updated_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, scan_results_updated_event);
                }
            },
            EngineEvent::TrackableTask(trackable_task_event) => match trackable_task_event {
                TrackableTaskEvent::ProgressChanged { progress_changed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, progress_changed_event);
                }
            },
        }
    }

    fn refresh_symbol_registry_snapshot(self: &Arc<Self>) {
        let registry_get_snapshot_request = RegistryGetSnapshotRequest::default();
        let engine_unprivileged_state = self.clone();

        let _ = registry_get_snapshot_request.send(self, move |registry_get_snapshot_response| {
            engine_unprivileged_state.apply_symbol_registry_snapshot(registry_get_snapshot_response.symbol_registry_snapshot);
        });
    }

    fn ensure_symbol_registry_snapshot_current(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> bool {
        let current_generation = self
            .symbol_registry_mirror
            .read()
            .map(|symbol_registry_mirror| symbol_registry_mirror.get_generation())
            .unwrap_or_default();

        if current_generation >= expected_generation {
            return true;
        }

        let registry_get_snapshot_request = RegistryGetSnapshotRequest::default();
        let engine_unprivileged_state = self.clone();
        let (completion_sender, completion_receiver) = bounded(1);
        let did_send = registry_get_snapshot_request.send(self, move |registry_get_snapshot_response| {
            let symbol_registry_snapshot = registry_get_snapshot_response.symbol_registry_snapshot;
            let applied_generation = symbol_registry_snapshot.get_generation();

            engine_unprivileged_state.apply_symbol_registry_snapshot(symbol_registry_snapshot);
            let _ = completion_sender.send(applied_generation);
        });

        if !did_send {
            log::error!(
                "Failed to dispatch registry snapshot refresh while waiting for generation {}.",
                expected_generation
            );
            return false;
        }

        match completion_receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(applied_generation) => applied_generation >= expected_generation,
            Err(error) => {
                log::error!(
                    "Timed out waiting for symbol registry mirror to reach generation {}: {}",
                    expected_generation,
                    error
                );
                false
            }
        }
    }

    fn apply_symbol_registry_snapshot(
        &self,
        symbol_registry_snapshot: squalr_engine_api::registries::symbols::symbol_registry_snapshot::SymbolRegistrySnapshot,
    ) {
        if let Ok(mut symbol_registry_mirror) = self.symbol_registry_mirror.write() {
            symbol_registry_mirror.apply_snapshot(symbol_registry_snapshot.clone());
        } else {
            log::error!("Failed to acquire symbol registry mirror write lock while applying snapshot.");
        }

        self.symbol_registry_compatibility
            .apply_snapshot(&symbol_registry_snapshot);
    }

    /// Dispatches a particular engine event to all listeners for its event type.
    fn dispatch_engine_event<E: 'static + Any + Send + Sync>(
        event_listeners: &Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>>,
        event: E,
    ) {
        match event_listeners.read() {
            Ok(event_listeners_guard) => {
                if let Some(callbacks) = event_listeners_guard.get(&TypeId::of::<E>()) {
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
