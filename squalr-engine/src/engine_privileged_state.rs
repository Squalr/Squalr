use crate::engine_mode::EngineMode;
use crate::events::engine_event_handler::EngineEventHandler;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::events::process::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::structures::processes::process_info::OpenedProcessInfo;
use squalr_engine_api::structures::tasks::engine_trackable_task_handle::EngineTrackableTaskHandle;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

/// Tracks critical engine state for internal use. This includes executing engine tasks, commands, and events.
pub struct EnginePrivilegedState {
    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, including any scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// The event handler for listening to events emitted from the engine.
    event_handler: EngineEventHandler,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,
}

impl EnginePrivilegedState {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let mut optional_shell = None;

        if engine_mode == EngineMode::PrivilegedShell {
            optional_shell = Some(Arc::new(InterProcessPrivilegedShell::new()));
        }

        let execution_context = Arc::new(EnginePrivilegedState {
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new())),
            event_handler: EngineEventHandler::new(optional_shell),
            task_manager: TrackableTaskManager::new(),
        });

        execution_context.initialize();

        execution_context
    }

    fn initialize(self: &Arc<Self>) {
        self.event_handler.initialize(self);
    }

    pub fn set_opened_process(
        &self,
        process_info: OpenedProcessInfo,
    ) {
        if let Ok(mut process) = self.opened_process.write() {
            log::info!("Opened process: {}, pid: {}", process_info.name, process_info.process_id);
            *process = Some(process_info.clone());

            self.emit_event(EngineEvent::Process(ProcessChangedEvent {
                process_info: Some(process_info),
            }));
        }
    }

    pub fn clear_opened_process(&self) {
        if let Ok(mut process) = self.opened_process.write() {
            *process = None;
            log::info!("Process closed");
            self.emit_event(EngineEvent::Process(ProcessChangedEvent { process_info: None }));
        }
    }

    pub fn get_opened_process(&self) -> Option<OpenedProcessInfo> {
        match self.opened_process.read() {
            Ok(opened_process) => opened_process.clone(),
            Err(err) => {
                log::error!("Failed to access opened process: {}", err);
                None
            }
        }
    }

    pub fn get_snapshot(&self) -> Arc<RwLock<Snapshot>> {
        self.snapshot.clone()
    }

    /// Emits an event from the engine. Direct usage is not advised except by the engine code itself.
    pub fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        self.event_handler.subscribe()
    }

    /// Emits an event from the engine. Direct usage is not advised except by the engine code itself.
    pub fn emit_event(
        &self,
        event: EngineEvent,
    ) {
        self.event_handler.emit_event(event);
    }

    pub fn register_task(
        &self,
        trackable_task_handle: EngineTrackableTaskHandle,
    ) {
        self.task_manager.register_task(trackable_task_handle);
    }

    pub fn unregister_task(
        &self,
        task_identifier: &String,
    ) {
        self.task_manager.unregister_task(task_identifier);
    }
}
