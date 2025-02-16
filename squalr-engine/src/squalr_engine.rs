use crate::{command_dispatchers::command_dispatcher::CommandDispatcher, engine_mode::EngineMode};
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

    /// The process to which Squalr is attached.
    opened_process: RwLock<Option<OpenedProcessInfo>>,

    /// The current snapshot of process memory, which may contain previous and current scan results.
    snapshot: Arc<RwLock<Snapshot>>,
}

impl SqualrEngine {
    fn new(engine_mode: EngineMode) -> Self {
        // Initialize the command dispatcher, which handles routing and executing all engine commands.
        CommandDispatcher::initialize(engine_mode);

        SqualrEngine {
            engine_mode,
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

    pub fn set_opened_process(process_info: OpenedProcessInfo) {
        if let Ok(mut process) = Self::get_instance().opened_process.write() {
            Logger::get_instance().log(
                LogLevel::Info,
                &format!("Opened process: {}, pid: {}", process_info.name, process_info.process_id),
                None,
            );
            *process = Some(process_info.clone());
        }
    }

    pub fn clear_opened_process() {
        if let Ok(mut process) = Self::get_instance().opened_process.write() {
            *process = None;
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
