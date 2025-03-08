use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_architecture::vectors::Vectors;
use squalr_engine_common::logging::file_system_logger::FileSystemLogger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    // The global instance for all engine state, such as snapshots, scan results, running tasks, etc.
    _engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    // Execution context that wraps privileged state behind a publically usable API.
    engine_execution_context: Arc<EngineExecutionContext>,

    // Routes logs to the file system, as well as any optional subscribers to log events, such as output in the GUI.
    file_system_logger: Arc<FileSystemLogger>,
}

impl SqualrEngine {
    pub fn new(engine_mode: EngineMode) -> Self {
        let mut engine_privileged_state = None;

        if engine_mode == EngineMode::PrivilegedShell || engine_mode == EngineMode::Standalone {
            engine_privileged_state = Some(EnginePrivilegedState::new(engine_mode));
        }

        let engine_execution_context = EngineExecutionContext::new(engine_mode, engine_privileged_state.clone());

        let squalr_engine = SqualrEngine {
            _engine_privileged_state: engine_privileged_state,
            engine_execution_context,
            file_system_logger: Arc::new(FileSystemLogger::new()),
        };

        squalr_engine.initialize(engine_mode);

        squalr_engine
    }

    fn initialize(
        &self,
        engine_mode: EngineMode,
    ) {
        log::info!("Squalr started");
        log::info!(
            "CPU vector size for accelerated scans: {:?} bytes ({:?} bits), architecture: {}",
            Vectors::get_hardware_vector_size(),
            Vectors::get_hardware_vector_size() * 8,
            Vectors::get_hardware_vector_name(),
        );

        match engine_mode {
            EngineMode::Standalone | EngineMode::PrivilegedShell => {
                if let Err(err) = ProcessQuery::start_monitoring() {
                    log::error!("Failed to monitor system processes: {}", err);
                }
            }
            EngineMode::UnprivilegedHost => {}
        }
    }

    pub fn get_engine_execution_context(&self) -> &Arc<EngineExecutionContext> {
        &self.engine_execution_context
    }

    pub fn get_logger(&self) -> &Arc<FileSystemLogger> {
        &self.file_system_logger
    }
}
