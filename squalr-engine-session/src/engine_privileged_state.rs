use crate::os::ProcessManager;
use crate::os::engine_os_provider::EngineOsProviders;
use crate::registries::registries::Registries;
use crate::tasks::snapshot_scan_result_freeze_task::SnapshotScanResultFreezeTask;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_event_envelope::EngineEventEnvelope;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::events::registry::changed::registry_changed_event::RegistryChangedEvent;
use squalr_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use squalr_engine_api::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use squalr_engine_api::registries::registry_context::RegistryContext;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::registries::symbols::symbol_registry_snapshot::SymbolRegistrySnapshot;
use squalr_engine_api::registries::symbols::{data_type_descriptor::DataTypeDescriptor, symbolic_struct_descriptor::SymbolicStructDescriptor};
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_operating_system::process_query::process_query_error::ProcessQueryError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

/// Tracks critical privileged engine session state for command execution and event dispatch.
pub struct EnginePrivilegedState {
    /// The manager for the process to which Squalr is attached, and detecting if that process dies.
    process_manager: ProcessManager,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,

    /// The current snapshot of process memory, including any scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// The active pointer scan session and tree state.
    pointer_scan_session: Arc<RwLock<Option<PointerScanSession>>>,

    /// Monotonically increasing identifier for new pointer scan sessions.
    next_pointer_scan_session_id: AtomicU64,

    /// Monotonically increasing generation for symbol registry snapshots.
    symbol_registry_generation: AtomicU64,

    /// Serializes symbol registry mutation helpers so external code cannot race multi-step registry updates.
    symbol_registry_mutation_guard: Mutex<()>,

    /// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
    engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>>,

    /// The collection of all engine registries.
    registries: Arc<Registries>,

    /// OS access providers for process and memory operations.
    os_providers: EngineOsProviders,
}

impl EnginePrivilegedState {
    pub fn new(
        engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        os_providers: EngineOsProviders,
    ) -> Result<Arc<Self>, ProcessQueryError> {
        let event_emitter = Self::create_event_emitter(engine_bindings.clone());
        let process_manager = ProcessManager::new(event_emitter.clone());
        let task_manager = TrackableTaskManager::new();
        let snapshot = Arc::new(RwLock::new(Snapshot::new()));
        let pointer_scan_session = Arc::new(RwLock::new(None));
        let registries = Arc::new(Registries::new());

        SnapshotScanResultFreezeTask::start_task(
            process_manager.get_opened_process_ref(),
            registries.get_freeze_list_registry().clone(),
            os_providers.clone(),
        );

        let engine_privileged_state = Arc::new(EnginePrivilegedState {
            process_manager,
            task_manager,
            snapshot,
            pointer_scan_session,
            next_pointer_scan_session_id: AtomicU64::new(0),
            symbol_registry_generation: AtomicU64::new(1),
            symbol_registry_mutation_guard: Mutex::new(()),
            engine_bindings,
            registries,
            os_providers,
        });

        engine_privileged_state
            .os_providers
            .process_query
            .start_monitoring()?;

        Ok(engine_privileged_state)
    }

    /// Gets the process manager for this session.
    pub fn get_process_manager(&self) -> &ProcessManager {
        &self.process_manager
    }

    pub fn get_trackable_task_manager(&self) -> &TrackableTaskManager {
        &self.task_manager
    }

    /// Gets the current snapshot, which contains all captured memory and scan results.
    pub fn get_snapshot(&self) -> Arc<RwLock<Snapshot>> {
        self.snapshot.clone()
    }

    /// Gets the active pointer scan session, if any.
    pub fn get_pointer_scan_session(&self) -> Arc<RwLock<Option<PointerScanSession>>> {
        self.pointer_scan_session.clone()
    }

    /// Allocates a stable identifier for a new pointer scan session.
    pub fn allocate_pointer_scan_session_id(&self) -> u64 {
        self.next_pointer_scan_session_id.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn get_symbol_registry_snapshot(&self) -> SymbolRegistrySnapshot {
        let current_generation = self.symbol_registry_generation.load(Ordering::SeqCst);

        self.registries
            .get_symbol_registry()
            .create_snapshot(current_generation)
    }

    pub fn notify_symbol_registry_changed(&self) {
        let next_generation = self.symbol_registry_generation.fetch_add(1, Ordering::SeqCst) + 1;

        self.emit_event(RegistryChangedEvent { generation: next_generation });
    }

    pub fn get_symbol_registry_generation(&self) -> u64 {
        self.symbol_registry_generation.load(Ordering::SeqCst)
    }

    pub fn register_symbol_data_type_descriptor(
        &self,
        data_type_descriptor: DataTypeDescriptor,
    ) -> bool {
        self.mutate_symbol_registry(move |symbol_registry| symbol_registry.register_data_type_descriptor(data_type_descriptor))
    }

    pub fn unregister_symbol_data_type_descriptor(
        &self,
        data_type_id: &str,
    ) -> bool {
        self.mutate_symbol_registry(|symbol_registry| symbol_registry.unregister_data_type_descriptor(data_type_id))
    }

    pub fn register_symbolic_struct_descriptor(
        &self,
        symbolic_struct_descriptor: SymbolicStructDescriptor,
    ) -> bool {
        self.mutate_symbol_registry(move |symbol_registry| {
            symbol_registry.register_symbolic_struct(
                symbolic_struct_descriptor.get_symbolic_struct_id().to_string(),
                symbolic_struct_descriptor
                    .get_symbolic_struct_definition()
                    .clone(),
            )
        })
    }

    pub fn unregister_symbolic_struct_descriptor(
        &self,
        symbolic_struct_id: &str,
    ) -> bool {
        self.mutate_symbol_registry(|symbol_registry| symbol_registry.unregister_symbolic_struct(symbolic_struct_id))
    }

    pub fn set_project_symbol_catalog(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> bool {
        self.mutate_symbol_registry(|symbol_registry| symbol_registry.set_project_symbol_catalog(project_symbol_catalog.get_symbolic_struct_descriptors()))
    }

    /// Gets all engine registries.
    pub fn get_registries(&self) -> Arc<Registries> {
        self.registries.clone()
    }

    /// Gets OS providers used for process and memory operations.
    pub fn get_os_providers(&self) -> &EngineOsProviders {
        &self.os_providers
    }

    /// Gets the registry for the list of addresses that have been marked as frozen.
    pub fn get_freeze_list_registry(&self) -> Arc<RwLock<FreezeListRegistry>> {
        self.registries.get_freeze_list_registry()
    }

    /// Provides controlled read access to the symbol registry without exposing its handle publicly.
    pub fn read_symbol_registry<T>(
        &self,
        reader: impl FnOnce(&SymbolRegistry) -> T,
    ) -> T {
        reader(self.registries.get_symbol_registry().as_ref())
    }

    /// Gets the registry for project item types.
    pub fn get_project_item_type_registry(&self) -> Arc<RwLock<ProjectItemTypeRegistry>> {
        self.registries.get_project_item_type_registry()
    }

    /// Gets the registry for element scan rules.
    pub fn get_element_scan_rule_registry(&self) -> Arc<RwLock<ElementScanRuleRegistry>> {
        self.registries.get_element_scan_rule_registry()
    }

    /// Dispatches an event from the engine.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => engine_bindings.subscribe_to_engine_events(),
            Err(error) => Err(EngineBindingError::lock_failure(
                "subscribing to engine events from privileged state",
                error.to_string(),
            )),
        }
    }

    pub fn get_engine_bindings(&self) -> &Arc<RwLock<dyn EngineApiPrivilegedBindings>> {
        &self.engine_bindings
    }

    /// Dispatches an event from the engine.
    pub fn emit_event<F>(
        &self,
        engine_event: F,
    ) where
        F: EngineEventRequest,
    {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(error) = engine_bindings.emit_event(engine_event.to_engine_event()) {
                    log::error!("Error dispatching engine event: {}", error);
                }
            }
            Err(error) => {
                log::error!("Failed to acquire privileged engine bindings read lock: {}", error);
            }
        }
    }

    fn create_event_emitter(engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>>) -> Arc<dyn Fn(EngineEvent) + Send + Sync> {
        let engine_bindings = engine_bindings.clone();
        Arc::new(move |event: EngineEvent| {
            if let Ok(bindings) = engine_bindings.read() {
                if let Err(error) = bindings.emit_event(event) {
                    log::error!("Error dispatching engine event: {}", error);
                }
            }
        }) as Arc<dyn Fn(EngineEvent) + Send + Sync>
    }

    fn mutate_symbol_registry<F>(
        &self,
        mutator: F,
    ) -> bool
    where
        F: FnOnce(&SymbolRegistry) -> bool,
    {
        let mutation_guard = match self.symbol_registry_mutation_guard.lock() {
            Ok(mutation_guard) => mutation_guard,
            Err(error) => {
                log::error!("Failed to acquire symbol registry mutation guard: {}", error);
                return false;
            }
        };
        let did_change = mutator(self.registries.get_symbol_registry().as_ref());

        drop(mutation_guard);

        if did_change {
            self.notify_symbol_registry_changed();
        }

        did_change
    }
}

#[cfg(test)]
mod tests {
    use super::EnginePrivilegedState;
    use crate::os::engine_os_provider::{EngineOsProviders, ProcessQueryProvider};
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
    use squalr_engine_api::engine::{
        engine_api_priviliged_bindings::EngineApiPrivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
    };
    use squalr_engine_api::events::{engine_event::EngineEvent, registry::registry_event::RegistryEvent};
    use squalr_engine_api::registries::symbols::{data_type_descriptor::DataTypeDescriptor, symbolic_struct_descriptor::SymbolicStructDescriptor};
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::endian::Endian,
        processes::{opened_process_info::OpenedProcessInfo, process_info::ProcessInfo},
        projects::project_symbol_catalog::ProjectSymbolCatalog,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use squalr_engine_operating_system::process_query::{process_query_error::ProcessQueryError, process_query_options::ProcessQueryOptions};
    use std::sync::{Arc, Mutex, RwLock};

    struct NoOpProcessQueryProvider;

    impl ProcessQueryProvider for NoOpProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Ok(())
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            vec![]
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not implemented in no-op provider"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Ok(())
        }
    }

    struct CapturingPrivilegedBindings {
        emitted_events: Arc<Mutex<Vec<EngineEvent>>>,
    }

    impl CapturingPrivilegedBindings {
        fn new() -> Self {
            Self {
                emitted_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn take_events(&self) -> Vec<EngineEvent> {
            self.emitted_events
                .lock()
                .map(|events| events.clone())
                .unwrap_or_default()
        }
    }

    impl EngineApiPrivilegedBindings for CapturingPrivilegedBindings {
        fn emit_event(
            &self,
            event: EngineEvent,
        ) -> Result<(), EngineBindingError> {
            self.emitted_events
                .lock()
                .map(|mut events| events.push(event))
                .map_err(|error| EngineBindingError::lock_failure("capturing emitted privileged engine event", error.to_string()))?;

            Ok(())
        }

        fn dispatch_internal_command(
            &self,
            _engine_command: PrivilegedCommand,
            _callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching internal commands in producer wiring tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    #[test]
    fn set_project_symbol_catalog_bumps_generation_and_emits_registry_changed_event() {
        let bindings = Arc::new(RwLock::new(CapturingPrivilegedBindings::new()));
        let engine_privileged_state = create_test_engine_privileged_state(bindings.clone());
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![SymbolicStructDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )]);

        assert_eq!(engine_privileged_state.get_symbol_registry_generation(), 1);
        assert!(engine_privileged_state.set_project_symbol_catalog(&project_symbol_catalog));
        assert_eq!(engine_privileged_state.get_symbol_registry_generation(), 2);
        assert!(
            engine_privileged_state
                .get_symbol_registry_snapshot()
                .get_symbolic_struct_descriptors()
                .iter()
                .any(|symbolic_struct_descriptor| symbolic_struct_descriptor.get_symbolic_struct_id() == "player.stats")
        );

        let emitted_events = bindings
            .read()
            .map(|bindings| bindings.take_events())
            .unwrap_or_default();

        assert_eq!(emitted_events.len(), 1);
        assert!(matches!(
            emitted_events.first(),
            Some(EngineEvent::Registry(RegistryEvent::Changed { registry_changed_event })) if registry_changed_event.generation == 2
        ));
    }

    #[test]
    fn register_symbol_data_type_descriptor_bumps_generation_and_updates_snapshot() {
        let bindings = Arc::new(RwLock::new(CapturingPrivilegedBindings::new()));
        let engine_privileged_state = create_test_engine_privileged_state(bindings.clone());

        assert_eq!(engine_privileged_state.get_symbol_registry_generation(), 1);
        assert!(engine_privileged_state.register_symbol_data_type_descriptor(DataTypeDescriptor::new(
            String::from("remote.plugin.u24"),
            String::from("remote-icon"),
            3,
            vec![AnonymousValueStringFormat::Hexadecimal],
            AnonymousValueStringFormat::Hexadecimal,
            Endian::Little,
            false,
            false,
        )));
        assert_eq!(engine_privileged_state.get_symbol_registry_generation(), 2);
        assert!(
            engine_privileged_state
                .get_symbol_registry_snapshot()
                .get_data_type_descriptors()
                .iter()
                .any(|data_type_descriptor| data_type_descriptor.get_data_type_id() == "remote.plugin.u24")
        );

        let emitted_events = bindings
            .read()
            .map(|bindings| bindings.take_events())
            .unwrap_or_default();

        assert_eq!(emitted_events.len(), 1);
        assert!(matches!(
            emitted_events.first(),
            Some(EngineEvent::Registry(RegistryEvent::Changed { registry_changed_event })) if registry_changed_event.generation == 2
        ));
    }

    fn create_test_engine_privileged_state(bindings: Arc<RwLock<CapturingPrivilegedBindings>>) -> Arc<EnginePrivilegedState> {
        let mut engine_os_providers = EngineOsProviders::default();
        engine_os_providers.process_query = Arc::new(NoOpProcessQueryProvider);

        EnginePrivilegedState::new(bindings, engine_os_providers).expect("Expected the test engine privileged state to initialize.")
    }
}
