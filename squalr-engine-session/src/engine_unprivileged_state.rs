use crate::logging::log_dispatcher::{LogDispatcher, LogDispatcherOptions};
use crate::plugins::plugin_registry::PluginRegistry;
use crate::registries::privileged_registry_cache::PrivilegedRegistryCache;
use crate::virtual_snapshots::{
    virtual_snapshot::VirtualSnapshot, virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_resolver::materialize_virtual_snapshot_queries,
};
use crossbeam_channel::bounded;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::registry::get_metadata::registry_get_metadata_request::RegistryGetMetadataRequest;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::events::plugins::plugins_event::PluginsEvent;
use squalr_engine_api::events::process::process_event::ProcessEvent;
use squalr_engine_api::events::project::project_event::ProjectEvent;
use squalr_engine_api::events::project_items::project_items_event::ProjectItemsEvent;
use squalr_engine_api::events::registry::registry_event::RegistryEvent;
use squalr_engine_api::events::scan_results::scan_results_event::ScanResultsEvent;
use squalr_engine_api::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use squalr_engine_api::registries::symbols::privileged_registry_catalog::PrivilegedRegistryCatalog;
use squalr_engine_api::registries::symbols::symbol_registry_error::SymbolRegistryError;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue,
};
use squalr_engine_api::structures::projects::project_manager::ProjectManager;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
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
    /// Built-in plugin registry used by client-side extension points.
    plugin_registry: Arc<PluginRegistry>,
    /// Cached privileged-owned registry catalog synchronized from the engine.
    privileged_registry_cache: Arc<RwLock<PrivilegedRegistryCache>>,
    /// Session-owned virtual snapshots used by interactive views.
    virtual_snapshots: Arc<RwLock<HashMap<String, VirtualSnapshot>>>,
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
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_default_value(data_type_ref))
            .unwrap_or_default()
    }

    fn resolve_struct_layout_definition(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition> {
        self.resolve_local_project_struct_layout_definition(symbolic_struct_ref_id)
            .or_else(|| {
                self.read_privileged_registry_cache(|privileged_registry_cache| {
                    privileged_registry_cache.resolve_struct_layout_definition(symbolic_struct_ref_id)
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
        let plugin_registry = Arc::new(PluginRegistry::new());

        Arc::new(EngineUnprivilegedState {
            engine_api_unprivileged_bindings,
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
            file_system_logger: Arc::new(LogDispatcher::new_with_options(LogDispatcherOptions {
                enable_console_output: options.enable_console_logging,
            })),
            project_manager,
            plugin_registry,
            privileged_registry_cache: Arc::new(RwLock::new(PrivilegedRegistryCache::default())),
            virtual_snapshots: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn initialize(self: &Arc<Self>) {
        self.start_event_dispatcher();
        self.refresh_privileged_registry_catalog();
    }

    /// Gets the file system logger that routes log events to the log file.
    pub fn get_logger(&self) -> &Arc<LogDispatcher> {
        &self.file_system_logger
    }

    pub fn get_plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry.clone()
    }

    pub fn get_privileged_registry_generation(&self) -> u64 {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_generation())
            .unwrap_or_default()
    }

    pub fn get_privileged_registry_catalog(&self) -> Option<PrivilegedRegistryCatalog> {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_registry_catalog().cloned())
            .flatten()
    }

    pub fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_registered_data_type_refs())
            .unwrap_or_default()
    }

    pub fn is_registered_data_type_ref(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.is_registered_data_type_ref(data_type_ref))
            .unwrap_or(false)
    }

    pub fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_default_anonymous_value_string_format(data_type_ref))
            .unwrap_or_default()
    }

    pub fn get_supported_anonymous_value_string_formats(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<AnonymousValueStringFormat> {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.get_supported_anonymous_value_string_formats(data_type_ref))
            .unwrap_or_default()
    }

    pub fn resolve_supported_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
        preferred_format: AnonymousValueStringFormat,
    ) -> AnonymousValueStringFormat {
        let supported_formats = self.get_supported_anonymous_value_string_formats(data_type_ref);

        if supported_formats.is_empty() || supported_formats.contains(&preferred_format) {
            return preferred_format;
        }

        let default_format = self.get_default_anonymous_value_string_format(data_type_ref);

        if supported_formats.contains(&default_format) {
            return default_format;
        }

        supported_formats.first().copied().unwrap_or(preferred_format)
    }

    pub fn normalize_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &mut AnonymousValueString,
    ) -> bool {
        let resolved_format = self.resolve_supported_anonymous_value_string_format(data_type_ref, anonymous_value_string.get_anonymous_value_string_format());

        if resolved_format == anonymous_value_string.get_anonymous_value_string_format() {
            return false;
        }

        anonymous_value_string.set_anonymous_value_string_format(resolved_format);

        true
    }

    pub fn validate_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.validate_value_string(data_type_ref, anonymous_value_string))
            .unwrap_or(false)
    }

    pub fn validate_scan_constraint(
        &self,
        data_type_ref: &DataTypeRef,
        scan_compare_type: ScanCompareType,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.read_privileged_registry_cache(|privileged_registry_cache| {
            privileged_registry_cache.validate_scan_constraint(data_type_ref, scan_compare_type, anonymous_value_string)
        })
        .unwrap_or(false)
    }

    pub fn validate_scan_constraint_with_hex_pattern_matching(
        &self,
        data_type_ref: &DataTypeRef,
        scan_compare_type: ScanCompareType,
        anonymous_value_string: &AnonymousValueString,
        use_hex_pattern_matching: bool,
    ) -> bool {
        self.read_privileged_registry_cache(|privileged_registry_cache| {
            privileged_registry_cache.validate_scan_constraint_with_hex_pattern_matching(
                data_type_ref,
                scan_compare_type,
                anonymous_value_string,
                use_hex_pattern_matching,
            )
        })
        .unwrap_or(false)
    }

    pub fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, SymbolRegistryError> {
        self.read_privileged_registry_cache(|privileged_registry_cache| {
            privileged_registry_cache.deanonymize_value_string(data_type_ref, anonymous_value_string)
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
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.anonymize_value(data_value, anonymous_value_string_format))
            .unwrap_or_else(|| Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())))
    }

    pub fn anonymize_value_to_supported_formats(
        &self,
        data_value: &DataValue,
    ) -> Result<Vec<AnonymousValueString>, SymbolRegistryError> {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.anonymize_value_to_supported_formats(data_value))
            .unwrap_or_else(|| Err(SymbolRegistryError::data_type_not_registered("anonymize value", data_value.get_data_type_id())))
    }

    pub fn supports_scalar_integer_values(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.supports_scalar_integer_values(data_type_ref))
            .unwrap_or(false)
    }

    pub fn read_scalar_integer_value(
        &self,
        data_value: &DataValue,
    ) -> Result<Option<i128>, SymbolRegistryError> {
        self.read_privileged_registry_cache(|privileged_registry_cache| privileged_registry_cache.read_scalar_integer_value(data_value))
            .unwrap_or_else(|| {
                Err(SymbolRegistryError::data_type_not_registered(
                    "read scalar integer value",
                    data_value.get_data_type_id(),
                ))
            })
    }

    pub fn set_virtual_snapshot_queries(
        &self,
        virtual_snapshot_id: &str,
        refresh_interval: Duration,
        queries: Vec<VirtualSnapshotQuery>,
    ) {
        match self.virtual_snapshots.write() {
            Ok(mut virtual_snapshots) => {
                let virtual_snapshot = virtual_snapshots
                    .entry(virtual_snapshot_id.to_string())
                    .or_insert_with(|| VirtualSnapshot::new(refresh_interval));

                virtual_snapshot.set_refresh_interval(refresh_interval);
                virtual_snapshot.set_queries(queries);
            }
            Err(error) => {
                log::error!("Failed to acquire virtual snapshots write lock while setting queries: {}", error);
            }
        }
    }

    pub fn get_virtual_snapshot(
        &self,
        virtual_snapshot_id: &str,
    ) -> Option<VirtualSnapshot> {
        match self.virtual_snapshots.read() {
            Ok(virtual_snapshots) => virtual_snapshots.get(virtual_snapshot_id).cloned(),
            Err(error) => {
                log::error!("Failed to acquire virtual snapshots read lock while reading snapshot: {}", error);
                None
            }
        }
    }

    pub fn request_virtual_snapshot_refresh(
        self: &Arc<Self>,
        virtual_snapshot_id: &str,
    ) {
        let (queries, refresh_query_version) = match self.virtual_snapshots.write() {
            Ok(mut virtual_snapshots) => {
                let Some(virtual_snapshot) = virtual_snapshots.get_mut(virtual_snapshot_id) else {
                    return;
                };
                let now = Instant::now();

                if !virtual_snapshot.should_refresh(now) {
                    return;
                }

                let refresh_query_version = virtual_snapshot.mark_refresh_started(now);

                (virtual_snapshot.get_queries().to_vec(), refresh_query_version)
            }
            Err(error) => {
                log::error!("Failed to acquire virtual snapshots write lock while requesting refresh: {}", error);
                return;
            }
        };
        let engine_unprivileged_state = self.clone();
        let virtual_snapshot_id = virtual_snapshot_id.to_string();

        std::thread::spawn(move || {
            let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
            let query_results = materialize_virtual_snapshot_queries(&engine_execution_context, &queries);

            match engine_unprivileged_state.virtual_snapshots.write() {
                Ok(mut virtual_snapshots) => {
                    if let Some(virtual_snapshot) = virtual_snapshots.get_mut(&virtual_snapshot_id) {
                        virtual_snapshot.apply_refresh_results(refresh_query_version, query_results, Instant::now());
                    }
                }
                Err(error) => {
                    log::error!("Failed to acquire virtual snapshots write lock while applying refresh results: {}", error);
                }
            }
        });
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

    /// Dispatches an unprivileged command to the local execution layer.
    pub fn dispatch_unprivileged_command<F>(
        self: &Arc<Self>,
        unprivileged_command: UnprivilegedCommand,
        callback: F,
    ) where
        F: FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static,
    {
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.clone();

        match self.engine_api_unprivileged_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(error) = engine_bindings.dispatch_unprivileged_command(unprivileged_command, &engine_execution_context, Box::new(callback)) {
                    log::error!("Error dispatching unprivileged engine command: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire unprivileged engine bindings read lock for unprivileged commands: {}", error);
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
        if !engine_unprivileged_state.ensure_privileged_registry_catalog_current(engine_event_envelope.get_registry_generation()) {
            log::error!(
                "Failed to refresh privileged registry cache to generation {} before dispatching engine event.",
                engine_event_envelope.get_registry_generation()
            );
        }

        let engine_event = engine_event_envelope.into_engine_event();

        match engine_event {
            EngineEvent::Plugins(plugins_event) => match plugins_event {
                PluginsEvent::PluginsChanged { plugins_changed_event } => {
                    Self::dispatch_engine_event(&engine_unprivileged_state.event_listeners, plugins_changed_event);
                }
            },
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

    fn refresh_privileged_registry_catalog(self: &Arc<Self>) {
        let registry_get_metadata_request = RegistryGetMetadataRequest::default();
        let engine_unprivileged_state = self.clone();

        let _ = registry_get_metadata_request.send(self, move |registry_get_metadata_response| {
            engine_unprivileged_state.apply_privileged_registry_catalog(registry_get_metadata_response.privileged_registry_catalog);
        });
    }

    fn ensure_privileged_registry_catalog_current(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> bool {
        let current_generation = self.get_privileged_registry_generation();

        if current_generation >= expected_generation {
            return true;
        }

        let registry_get_metadata_request = RegistryGetMetadataRequest::default();
        let engine_unprivileged_state = self.clone();
        let (completion_sender, completion_receiver) = bounded(1);
        let did_send = registry_get_metadata_request.send(self, move |registry_get_metadata_response| {
            let privileged_registry_catalog = registry_get_metadata_response.privileged_registry_catalog;
            let applied_generation = privileged_registry_catalog.get_generation();

            engine_unprivileged_state.apply_privileged_registry_catalog(privileged_registry_catalog);
            let _ = completion_sender.send(applied_generation);
        });

        if !did_send {
            log::error!(
                "Failed to dispatch registry metadata refresh while waiting for generation {}.",
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

    fn apply_privileged_registry_catalog(
        &self,
        privileged_registry_catalog: squalr_engine_api::registries::symbols::privileged_registry_catalog::PrivilegedRegistryCatalog,
    ) {
        if let Ok(mut privileged_registry_cache) = self.privileged_registry_cache.write() {
            privileged_registry_cache.apply_registry_catalog(privileged_registry_catalog);
        } else {
            log::error!("Failed to acquire privileged registry cache write lock while applying privileged registry catalog.");
        }
    }

    fn read_privileged_registry_cache<T>(
        &self,
        reader: impl FnOnce(&PrivilegedRegistryCache) -> T,
    ) -> Option<T> {
        match self.privileged_registry_cache.read() {
            Ok(privileged_registry_cache) => Some(reader(&privileged_registry_cache)),
            Err(error) => {
                log::error!("Failed to acquire privileged registry cache read lock: {}", error);
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
