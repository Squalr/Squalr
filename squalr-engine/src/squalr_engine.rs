use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_mode::EngineMode;
use squalr_engine_architecture::vectors;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    engine_execution_context: Arc<EngineExecutionContext>,
}

impl SqualrEngine {
    pub fn new(engine_mode: EngineMode) -> Self {
        let squalr_engine = SqualrEngine {
            engine_execution_context: EngineExecutionContext::new(engine_mode),
        };

        squalr_engine.initialize(engine_mode);

        squalr_engine
    }

    fn initialize(
        &self,
        engine_mode: EngineMode,
    ) {
        Logger::get_instance().log(LogLevel::Info, "Squalr started", None);
        vectors::log_vector_architecture();

        match engine_mode {
            EngineMode::Standalone | EngineMode::PrivilegedShell => {
                if let Err(err) = ProcessQuery::start_monitoring() {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to monitor system processes: {}", err), None);
                }
            }
            EngineMode::UnprivilegedHost => {}
        }
    }

    pub fn get_engine_execution_context(&self) -> &Arc<EngineExecutionContext> {
        &self.engine_execution_context
    }
}
