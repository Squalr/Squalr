use crate::command_executors::engine_command_dispatcher::EngineCommandDispatcher;
use crate::engine_mode::EngineMode;
use crate::events::engine_event_handler::EngineEventHandler;
use crate::tasks::trackable_task_manager::TrackableTaskManager;
use crossbeam_channel::Receiver;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use interprocess_shell::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use squalr_engine_api::commands::engine_response::EngineResponse;
use squalr_engine_api::events::process::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::{commands::engine_command::EngineCommand, events::engine_event::EngineEvent};
use squalr_engine_common::structures::process_info::OpenedProcessInfo;
use squalr_engine_common::tasks::trackable_task_handle::TrackableTaskHandle;
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

/// Tracks information vital to executing tasks, commands, and events in the engine.
pub struct EngineExecutionContext {
    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,

    /// Defines the mode in which the engine is running.
    /// - Standalone engine is self-handling. This is the most common way Squalr is used.
    /// - Unprivileged host sends data via ipc. Used on platforms like Android.
    /// - Privileged shell returns data via ipc. Used on platforms like Android.
    engine_mode: EngineMode,

    /// The dispatcher that sends commands to the engine.
    command_dispatcher: EngineCommandDispatcher,

    /// The event handler for listening to events emitted from the engine.
    event_handler: EngineEventHandler,

    /// The manager that tracks all running engine tasks.
    task_manager: TrackableTaskManager,
}

impl EngineExecutionContext {
    pub fn new(engine_mode: EngineMode) -> Arc<Self> {
        let mut optional_host = None;
        let mut optional_shell = None;

        if engine_mode == EngineMode::UnprivilegedHost {
            optional_host = Some(Arc::new(InterProcessUnprivilegedHost::new()));
        } else if engine_mode == EngineMode::PrivilegedShell {
            optional_shell = Some(Arc::new(InterProcessPrivilegedShell::new()));
        }

        let execution_context = Arc::new(EngineExecutionContext {
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
            engine_mode,
            command_dispatcher: EngineCommandDispatcher::new(optional_host),
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
        self.opened_process.read().ok().and_then(|guard| guard.clone())
    }

    pub fn get_snapshot(&self) -> Arc<RwLock<Snapshot>> {
        self.snapshot.clone()
    }

    pub fn get_engine_mode(&self) -> EngineMode {
        self.engine_mode
    }

    /// Dispatches a command to the engine. Direct usage is generally not advised unless you know what you are doing.
    /// Instead, create `{Command}Request` instances and call `.send()` directly on them.
    /// This is only made public to support direct usage by CLIs and other features that need direct access.
    pub fn dispatch_command<F>(
        self: &Arc<Self>,
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        self.command_dispatcher
            .dispatch_command(command, self, callback)
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
        trackable_task_handle: TrackableTaskHandle,
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
