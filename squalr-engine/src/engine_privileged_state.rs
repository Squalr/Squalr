use crate::engine_bindings::interprocess::interprocess_privileged_shell::InterprocessPrivilegedShell;
use crate::engine_bindings::{engine_priviliged_bindings::EnginePrivilegedBindings, standalone::standalone_privileged_engine::StandalonePrivilegedEngine};
use crate::engine_mode::EngineMode;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use squalr_engine_api::events::engine_event::{EngineEvent, EngineEventRequest};
use squalr_engine_api::events::process::changed::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use squalr_engine_projects::project::project::Project;
use squalr_engine_scanning::results::snapshot_scan_result_freeze_list::SnapshotScanResultFreezeList;
use squalr_engine_scanning::results::snapshot_scan_result_freeze_task::SnapshotScanResultFreezeTask;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// Tracks critical engine state for internal use. This includes executing engine tasks, commands, and events.
pub struct EnginePrivilegedState {
    /// The process to which Squalr is attached.
    opened_process: Arc<RwLock<Option<OpenedProcessInfo>>>,

    /// The current snapshot of process memory, including any scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    // The list of frozen scan results.
    snapshot_scan_result_freeze_list: Arc<RwLock<SnapshotScanResultFreezeList>>,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,

    /// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
    engine_bindings: Arc<RwLock<dyn EnginePrivilegedBindings>>,

    /// The current opened project.
    opened_project: Arc<RwLock<Option<Project>>>,
}

impl EnginePrivilegedState {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let engine_bindings: Arc<RwLock<dyn EnginePrivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandalonePrivilegedEngine::new())),
            EngineMode::PrivilegedShell => Arc::new(RwLock::new(InterprocessPrivilegedShell::new())),
            EngineMode::UnprivilegedHost => unreachable!("Privileged state should never be created on an unprivileged host."),
        };

        let snapshot = Arc::new(RwLock::new(Snapshot::new()));
        let opened_process = Arc::new(RwLock::new(None));
        let snapshot_scan_result_freeze_list = Arc::new(RwLock::new(SnapshotScanResultFreezeList::new()));
        let opened_project = Arc::new(RwLock::new(None));

        SnapshotScanResultFreezeTask::start_task(opened_process.clone(), snapshot_scan_result_freeze_list.clone());

        let execution_context = Arc::new(EnginePrivilegedState {
            opened_process,
            snapshot,
            snapshot_scan_result_freeze_list,
            task_manager: TrackableTaskManager::new(),
            engine_bindings,
            opened_project,
        });

        Self::listen_for_open_process_changes(execution_context.clone());

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

    /// Sets the process to which we are currently attached.
    pub fn set_opened_process(
        &self,
        process_info: OpenedProcessInfo,
    ) {
        if let Ok(mut process) = self.opened_process.write() {
            log::info!("Opened process: {}, pid: {}", process_info.get_name(), process_info.get_process_id());
            *process = Some(process_info.clone());

            self.emit_event(ProcessChangedEvent {
                process_info: Some(process_info),
            });
        }
    }

    /// Clears the process to which we are currently attached.
    pub fn clear_opened_process(&self) {
        if let Ok(mut process) = self.opened_process.write() {
            *process = None;
            log::info!("Process closed");
            self.emit_event(ProcessChangedEvent { process_info: None });
        }
    }

    /// Gets the process to which we are currently attached, if any.
    pub fn get_opened_process(&self) -> Option<OpenedProcessInfo> {
        match self.opened_process.read() {
            Ok(opened_process) => opened_process.clone(),
            Err(err) => {
                log::error!("Failed to access opened process: {}", err);
                None
            }
        }
    }

    /// Gets a ref that points to the process to which we are currently attached, if any.
    pub fn get_opened_process_ref(&self) -> Arc<RwLock<Option<OpenedProcessInfo>>> {
        self.opened_process.clone()
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

    /// Dispatches an event from the engine.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        match self.engine_bindings.read() {
            Ok(engine_bindings) => engine_bindings.subscribe_to_engine_events(),
            Err(err) => Err(format!("Failed to acquire privileged engine bindings read lock: {}", err)),
        }
    }

    pub fn get_trackable_task_manager(&self) -> &TrackableTaskManager {
        &self.task_manager
    }

    /// Listens for the death of the currently opened process by polling for it repeatedly.
    fn listen_for_open_process_changes(execution_context: Arc<EnginePrivilegedState>) {
        std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));

                let opened_process_id = {
                    if let Some(opened_process) = execution_context.get_opened_process().as_ref() {
                        opened_process.get_process_id()
                    } else {
                        continue;
                    }
                };

                let process_query_options = ProcessQueryOptions {
                    required_process_id: Some(opened_process_id),
                    search_name: None,
                    require_windowed: false,
                    match_case: false,
                    fetch_icons: false,
                    limit: Some(1),
                };

                let processes = ProcessQuery::get_processes(process_query_options);

                if processes.len() <= 0 {
                    execution_context.clear_opened_process();
                }
            }
        });
    }
}
