use crate::engine_bindings::interprocess::interprocess_privileged_shell::InterprocessPrivilegedShell;
use crate::engine_bindings::{engine_priviliged_bindings::EnginePrivilegedBindings, standalone::standalone_privileged_engine::StandalonePrivilegedEngine};
use crate::engine_mode::EngineMode;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_processes::process::process_manager::ProcessManager;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_projects::project::project_manager::ProjectManager;
use squalr_engine_scanning::results::snapshot_scan_result_freeze_list::SnapshotScanResultFreezeList;
use squalr_engine_scanning::results::snapshot_scan_result_freeze_task::SnapshotScanResultFreezeTask;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

/// Tracks critical engine state for internal use. This includes executing engine tasks, commands, and events.
pub struct EnginePrivilegedState {
    /// The manager for the opened project and projects list.
    project_manager: ProjectManager,

    /// The manager for the process to which Squalr is attached, and detecting if that process dies.
    process_manager: ProcessManager,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,

    /// The current snapshot of process memory, including any scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    // The list of frozen scan results.
    snapshot_scan_result_freeze_list: Arc<RwLock<SnapshotScanResultFreezeList>>,

    /// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
    engine_bindings: Arc<RwLock<dyn EnginePrivilegedBindings>>,
}

impl EnginePrivilegedState {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let engine_bindings: Arc<RwLock<dyn EnginePrivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandalonePrivilegedEngine::new())),
            EngineMode::PrivilegedShell => Arc::new(RwLock::new(InterprocessPrivilegedShell::new())),
            EngineMode::UnprivilegedHost => unreachable!("Privileged state should never be created on an unprivileged host."),
        };

        let event_emitter = Self::create_event_emitter(engine_bindings.clone());
        let process_manager = ProcessManager::new(event_emitter.clone());
        let project_manager = ProjectManager::new(event_emitter);
        let task_manager = TrackableTaskManager::new();
        let snapshot = Arc::new(RwLock::new(Snapshot::new()));
        let snapshot_scan_result_freeze_list = Arc::new(RwLock::new(SnapshotScanResultFreezeList::new()));

        SnapshotScanResultFreezeTask::start_task(process_manager.get_opened_process_ref(), snapshot_scan_result_freeze_list.clone());

        let execution_context = Arc::new(EnginePrivilegedState {
            process_manager,
            project_manager,
            task_manager,
            snapshot,
            snapshot_scan_result_freeze_list,
            engine_bindings,
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
                    log::error!("Error initializing privileged engine bindings: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire privileged engine bindings write lock: {}", err);
            }
        }

        if let Err(err) = ProcessQuery::start_monitoring() {
            log::error!("Failed to monitor system processes: {}", err);
        }
    }

    /// Gets the project manager for this session.
    pub fn get_project_manager(&self) -> &ProjectManager {
        &self.project_manager
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

    /// Gets the list of scan results that have been marked as frozen.
    pub fn get_snapshot_scan_result_freeze_list(&self) -> Arc<RwLock<SnapshotScanResultFreezeList>> {
        self.snapshot_scan_result_freeze_list.clone()
    }

    /// Dispatches an event from the engine.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => engine_bindings.subscribe_to_engine_events(),
            Err(err) => Err(format!("Failed to acquire privileged engine bindings read lock: {}", err)),
        }
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
                if let Err(err) = engine_bindings.emit_event(engine_event.to_engine_event()) {
                    log::error!("Error dispatching engine event: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to acquire privileged engine bindings read lock: {}", err);
            }
        }
    }

    fn create_event_emitter(engine_bindings: Arc<RwLock<dyn EnginePrivilegedBindings>>) -> Arc<dyn Fn(EngineEvent) + Send + Sync> {
        let engine_bindings = engine_bindings.clone();
        Arc::new(move |event: EngineEvent| {
            if let Ok(bindings) = engine_bindings.read() {
                if let Err(err) = bindings.emit_event(event) {
                    log::error!("Error dispatching engine event: {}", err);
                }
            }
        }) as Arc<dyn Fn(EngineEvent) + Send + Sync>
    }
}
