use crate::commands::engine_response::EngineResponse;
use crate::commands::{engine_command::EngineCommand, engine_command_dispatcher::EngineCommandDispatcher};
use crate::engine_mode::EngineMode;
use crate::events::engine_event::EngineEvent;
use crate::events::event_handler::EngineEventHandler;
use crate::events::process::process_changed_event::ProcessChangedEvent;
use crossbeam_channel::Receiver;
use interprocess_shell::shell::inter_process_privileged_shell::InterProcessPrivilegedShell;
use interprocess_shell::shell::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use squalr_engine_architecture::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::{process_info::OpenedProcessInfo, process_query::process_queryer::ProcessQuery};
use squalr_engine_scanning::snapshots::snapshot::Snapshot;
use std::sync::{Arc, Once, RwLock};

static mut INSTANCE: Option<SqualrEngine> = None;
static INIT: Once = Once::new();

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    /// Defines the mode in which the engine is running.
    /// - Standalone engine is self-handling. This is the most common way Squalr is used.
    /// - Unprivileged host sends data via ipc. Used on platforms like Android.
    /// - Privileged shell returns data via ipc. Used on platforms like Android.
    engine_mode: EngineMode,

    /// The dispatcher that sends commands to the engine.
    command_dispatcher: Arc<EngineCommandDispatcher>,

    /// The event handler for listening to events emitted from the engine.
    event_handler: Arc<EngineEventHandler>,

    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        let mut optional_host = None;
        let mut optional_shell = None;

        if engine_mode == EngineMode::UnprivilegedHost {
            optional_host = Some(Arc::new(InterProcessUnprivilegedHost::new()));
        } else if engine_mode == EngineMode::PrivilegedShell {
            optional_shell = Some(Arc::new(InterProcessPrivilegedShell::new()));
        }

        SqualrEngine {
            engine_mode,
            command_dispatcher: Arc::new(EngineCommandDispatcher::new(optional_host)),
            event_handler: Arc::new(EngineEventHandler::new(optional_shell)),
            opened_process: RwLock::new(None),
            snapshot: Arc::new(RwLock::new(Snapshot::new(vec![]))),
        }
    }

    fn create_instance(engine_mode: EngineMode) {
        unsafe {
            INIT.call_once(|| {
                INSTANCE = Some(SqualrEngine::new(engine_mode));
            });
        }
    }

    fn get_instance() -> &'static SqualrEngine {
        unsafe {
            // If create_instance() has never been called before, default to standalone.
            if !INIT.is_completed() {
                panic!("Attempted to use engine before it was initialized");
            }

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn initialize(engine_mode: EngineMode) {
        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        Self::create_instance(engine_mode);

        match engine_mode {
            EngineMode::Standalone | EngineMode::PrivilegedShell => {
                if let Err(err) = ProcessQuery::start_monitoring() {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
                }
            }
            EngineMode::UnprivilegedHost => {}
        }
    }

    pub fn get_engine_mode() -> EngineMode {
        Self::get_instance().engine_mode
    }

    /// Dispatches a command to the engine. Direct usage is generally not advised unless you know what you are doing.
    /// Instead, create `{Command}Request` instances and call `.send()` directly on them.
    /// This is only made public to support direct usage by CLIs and other features that need direct access.
    pub fn dispatch_command<F>(
        command: EngineCommand,
        callback: F,
    ) where
        F: FnOnce(EngineResponse) + Send + Sync + 'static,
    {
        Self::get_instance()
            .command_dispatcher
            .dispatch_command(command, callback)
    }

    /// Emits an event from the engine. Direct usage is not advised except by the engine code itself.
    pub fn subscribe_to_engine_events() -> Receiver<EngineEvent> {
        Self::get_instance().event_handler.subscribe()
    }

    /// Emits an event from the engine. Direct usage is not advised except by the engine code itself.
    pub fn emit_event(event: EngineEvent) {
        Self::get_instance().event_handler.emit_event(event);
    }

    pub fn set_opened_process(process_info: OpenedProcessInfo) {
        if let Ok(mut process) = Self::get_instance().opened_process.write() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Opened process: {}, pid: {}", process_info.name, process_info.process_id),
                None,
            );
            *process = Some(process_info.clone());

            Self::emit_event(EngineEvent::Process(ProcessChangedEvent {
                process_info: Some(process_info),
            }));
        }
    }

    pub fn clear_opened_process() {
        if let Ok(mut process) = Self::get_instance().opened_process.write() {
            *process = None;
            Self::emit_event(EngineEvent::Process(ProcessChangedEvent { process_info: None }));
            Logger::get_instance().log(LogLevel::Info, "Process closed", None);
        }
    }

    pub fn get_opened_process() -> Option<OpenedProcessInfo> {
        Self::get_instance()
            .opened_process
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn get_snapshot() -> Arc<RwLock<Snapshot>> {
        Self::get_instance().snapshot.clone()
    }
}
