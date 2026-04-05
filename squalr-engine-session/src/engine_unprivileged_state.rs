use crate::logging::log_dispatcher::{LogDispatcher, LogDispatcherOptions};
use crate::registries::privileged_symbol_catalog::PrivilegedSymbolCatalog;
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
use squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError;
use squalr_engine_api::registries::symbols::registry_metadata::RegistryMetadata;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue,
};
use squalr_engine_api::structures::projects::project_manager::ProjectManager;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
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
    /// Cached privileged-owned symbol metadata synchronized from the engine.
    privileged_symbol_catalog: Arc<RwLock<PrivilegedSymbolCatalog>>,
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
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_default_value(data_type_ref))
            .unwrap_or_default()
    }

    fn resolve_struct_layout_definition(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition> {
        self.resolve_local_project_struct_layout_definition(symbolic_struct_ref_id)
            .or_else(|| {
                self.read_privileged_symbol_catalog(|privileged_symbol_catalog| {
                    privileged_symbol_catalog.resolve_struct_layout_definition(symbolic_struct_ref_id)
                })
                .unwrap_or_default()
            })
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
            privileged_symbol_catalog: Arc::new(RwLock::new(PrivilegedSymbolCatalog::default())),
        })
    }

    pub fn initialize(self: &Arc<Self>) {
        self.start_event_dispatcher();
        self.refresh_privileged_registry_metadata();
    }

    /// Gets the file system logger that routes log events to the log file.
    pub fn get_logger(&self) -> &Arc<LogDispatcher> {
        &self.file_system_logger
    }

    pub fn get_privileged_registry_generation(&self) -> u64 {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_generation())
            .unwrap_or_default()
    }

    pub fn get_privileged_registry_metadata(&self) -> Option<RegistryMetadata> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_snapshot().cloned())
            .flatten()
    }

    pub fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_registered_data_type_refs())
            .unwrap_or_default()
    }

    pub fn is_registered_data_type_ref(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.is_registered_data_type_ref(data_type_ref))
            .unwrap_or(false)
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_default_anonymous_value_string_format(data_type_ref))
            .unwrap_or_default()
    }

    pub fn get_supported_anonymous_value_string_formats(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<AnonymousValueStringFormat> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.get_supported_anonymous_value_string_formats(data_type_ref))
            .unwrap_or_default()
    }

    pub fn validate_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.validate_value_string(data_type_ref, anonymous_value_string))
            .unwrap_or(false)
    }

    pub fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, SymbolRegistryError> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| {
            privileged_symbol_catalog.deanonymize_value_string(data_type_ref, anonymous_value_string)
        })
        .unwrap_or_else(|| {
            Err(SymbolRegistryError::data_type_not_registered(
                "deanonymize value string",
                data_type_ref.get_data_type_id(),
            ))
        })
    }

    pub fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, SymbolRegistryError> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.anonymize_value(data_value, anonymous_value_string_format))
            .unwrap_or_else(|| Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())))
    }

    pub fn anonymize_value_to_supported_formats(
        &self,
        data_value: &DataValue,
    ) -> Result<Vec<AnonymousValueString>, SymbolRegistryError> {
        self.read_privileged_symbol_catalog(|privileged_symbol_catalog| privileged_symbol_catalog.anonymize_value_to_supported_formats(data_value))
            .unwrap_or_else(|| Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())))
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
        if !engine_unprivileged_state.ensure_privileged_registry_metadata_current(engine_event_envelope.get_registry_generation()) {
            log::error!(
                "Failed to refresh privileged symbol catalog to generation {} before dispatching engine event.",
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

    fn refresh_privileged_registry_metadata(self: &Arc<Self>) {
        let registry_get_snapshot_request = RegistryGetSnapshotRequest::default();
        let engine_unprivileged_state = self.clone();

        let _ = registry_get_snapshot_request.send(self, move |registry_get_snapshot_response| {
            engine_unprivileged_state.apply_privileged_registry_metadata(registry_get_snapshot_response.registry_metadata);
        });
    }

    fn ensure_privileged_registry_metadata_current(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> bool {
        let current_generation = self.get_privileged_registry_generation();

        if current_generation >= expected_generation {
            return true;
        }

        let registry_get_snapshot_request = RegistryGetSnapshotRequest::default();
        let engine_unprivileged_state = self.clone();
        let (completion_sender, completion_receiver) = bounded(1);
        let did_send = registry_get_snapshot_request.send(self, move |registry_get_snapshot_response| {
            let registry_metadata = registry_get_snapshot_response.registry_metadata;
            let applied_generation = registry_metadata.get_generation();

            engine_unprivileged_state.apply_privileged_registry_metadata(registry_metadata);
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
                    "Timed out waiting for privileged symbol catalog to reach generation {}: {}",
                    expected_generation,
                    error
                );
                false
            }
        }
    }

    fn apply_privileged_registry_metadata(
        &self,
        registry_metadata: squalr_engine_api::registries::symbols::registry_metadata::RegistryMetadata,
    ) {
        if let Ok(mut privileged_symbol_catalog) = self.privileged_symbol_catalog.write() {
            privileged_symbol_catalog.apply_snapshot(registry_metadata);
        } else {
            log::error!("Failed to acquire privileged symbol catalog write lock while applying snapshot.");
        }
    }

    fn read_privileged_symbol_catalog<T>(
        &self,
        reader: impl FnOnce(&PrivilegedSymbolCatalog) -> T,
    ) -> Option<T> {
        match self.privileged_symbol_catalog.read() {
            Ok(privileged_symbol_catalog) => Some(reader(&privileged_symbol_catalog)),
            Err(error) => {
                log::error!("Failed to acquire privileged symbol catalog read lock: {}", error);
                None
            }
        }
    }

    fn resolve_local_project_struct_layout_definition(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition> {
        self.get_opened_project_symbol_catalog()
            .and_then(|project_symbol_catalog| {
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbolic_struct_ref_id)
                    .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_definition().clone())
            })
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self.project_manager.get_opened_project();

        match opened_project.read() {
            Ok(opened_project_guard) => opened_project_guard
                .as_ref()
                .map(|project| project.get_project_info().get_project_symbol_catalog().clone()),
            Err(error) => {
                log::error!("Failed to acquire opened project lock while reading project symbol catalog: {}", error);
                None
            }
        }
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
