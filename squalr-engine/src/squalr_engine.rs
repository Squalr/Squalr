use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_mode::EngineMode;
use squalr_engine_architecture::vectors::Vectors;
use squalr_engine_common::logging::file_system_logger::FileSystemLogger;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    engine_execution_context: Arc<EngineExecutionContext>,
    file_system_logger: Arc<FileSystemLogger>,
}

impl SqualrEngine {
    pub fn new(engine_mode: EngineMode) -> Self {
        let squalr_engine = SqualrEngine {
            engine_execution_context: EngineExecutionContext::new(engine_mode),
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
