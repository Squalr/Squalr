use crate::engine_bindings::interprocess::interprocess_engine_api_privileged_bindings::InterprocessEngineApiPrivilegedBindings;
use crate::engine_bindings::standalone::standalone_engine_api_privileged_bindings::StandalonePrivilegedEngine;
use crate::engine_mode::EngineMode;
use crate::os::engine_os_provider::EngineOsProviders;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::registries::freeze_list::freeze_list_registry::FreezeListRegistry;
use squalr_engine_api::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use squalr_engine_api::registries::registries::Registries;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_processes::process::process_manager::ProcessManager;
use squalr_engine_scanning::freeze_task::snapshot_scan_result_freeze_task::SnapshotScanResultFreezeTask;
use std::sync::{Arc, RwLock};

/// Tracks critical engine state for internal use. This includes executing engine tasks, commands, and events.
pub struct EnginePrivilegedState {
    /// The manager for the process to which Squalr is attached, and detecting if that process dies.
    process_manager: ProcessManager,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,

    /// The current snapshot of process memory, including any scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
    engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>>,

    /// The collection of all engine registries.
    registries: Arc<Registries>,

    /// OS access providers for process and memory operations.
    os_providers: EngineOsProviders,
}

impl EnginePrivilegedState {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        Self::new_with_os_providers(engine_mode, EngineOsProviders::default())
    }

    pub fn new_with_os_providers(
        engine_mode: EngineMode,
        os_providers: EngineOsProviders,
    ) -> Arc<Self> {
        let engine_bindings_standalone = match engine_mode {
            EngineMode::Standalone => Some(Arc::new(RwLock::new(StandalonePrivilegedEngine::new()))),
            _ => None,
        };
        let engine_bindings_interprocess = match engine_mode {
            EngineMode::PrivilegedShell => Some(Arc::new(RwLock::new(InterprocessEngineApiPrivilegedBindings::new()))),
            _ => None,
        };

        let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => unsafe { engine_bindings_standalone.clone().unwrap_unchecked() },
            EngineMode::PrivilegedShell => unsafe { engine_bindings_interprocess.clone().unwrap_unchecked() },
            EngineMode::UnprivilegedHost => unreachable!("Privileged state should never be created on an unprivileged host."),
        };

        let event_emitter = Self::create_event_emitter(engine_bindings.clone());
        let process_manager = ProcessManager::new(event_emitter.clone());
        let task_manager = TrackableTaskManager::new();
        let snapshot = Arc::new(RwLock::new(Snapshot::new()));
        let registries = Arc::new(Registries::new());

        SnapshotScanResultFreezeTask::start_task(process_manager.get_opened_process_ref(), registries.get_freeze_list_registry().clone());

        let engine_privileged_state = Arc::new(EnginePrivilegedState {
            process_manager,
            task_manager,
            snapshot,
            engine_bindings,
            registries,
            os_providers,
        });

        // Initialize standalone privileged bindings if they are present.
        if let Some(engine_bindings_standalone) = engine_bindings_standalone.as_ref() {
            match engine_bindings_standalone.write() {
                Ok(mut engine_bindings_standalone) => {
                    if let Err(error) = engine_bindings_standalone.initialize(&engine_privileged_state) {
                        log::error!("Error initializing standalone privileged engine bindings: {}", error);
                    }
                }
                Err(error) => {
                    log::error!("Failed to acquire standalone privileged engine bindings write lock: {}", error);
                }
            }
        }

        // Initialize interprocess privileged bindings if they are present.
        if let Some(engine_bindings_interprocess) = engine_bindings_interprocess.as_ref() {
            match engine_bindings_interprocess.write() {
                Ok(mut engine_bindings_interprocess) => {
                    if let Err(error) = engine_bindings_interprocess.initialize(&engine_privileged_state) {
                        log::error!("Error initializing interprocess privileged engine bindings: {}", error);
                    }
                }
                Err(error) => {
                    log::error!("Failed to acquire interprocess privileged engine bindings write lock: {}", error);
                }
            }
        }

        if let Err(error) = engine_privileged_state
            .os_providers
            .process_query
            .start_monitoring()
        {
            log::error!("Failed to monitor system processes: {}", error);
        }

        engine_privileged_state
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

    /// Gets the registry for symbols.
    pub fn get_symbol_registry(&self) -> Arc<RwLock<SymbolRegistry>> {
        self.registries.get_symbol_registry()
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
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => engine_bindings.subscribe_to_engine_events(),
            Err(error) => Err(format!("Failed to acquire privileged engine bindings read lock: {}", error)),
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
}
