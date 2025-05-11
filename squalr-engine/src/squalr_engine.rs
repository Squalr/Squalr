use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::dependency_injection::dependency_container_builder::DependencyContainerBuilder;
use squalr_engine_architecture::vectors::Vectors;
use std::sync::Arc;

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    // The global instance for all engine state, such as snapshots, scan results, running tasks, etc.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    // Execution context that wraps privileged state behind a publically usable API.
    engine_execution_context: Option<Arc<EngineExecutionContext>>,
}

impl SqualrEngine {
    pub fn new(
        engine_mode: EngineMode,
        dependency_container_builder: &mut DependencyContainerBuilder,
    ) -> anyhow::Result<Self> {
        let mut engine_privileged_state = None;
        let mut engine_execution_context = None;

        match engine_mode {
            EngineMode::Standalone => {
                engine_privileged_state = Some(EnginePrivilegedState::new(engine_mode));
                engine_execution_context = Some(EngineExecutionContext::new(engine_mode));
            }
            EngineMode::PrivilegedShell => {
                engine_privileged_state = Some(EnginePrivilegedState::new(engine_mode));
            }
            EngineMode::UnprivilegedHost => {
                engine_execution_context = Some(EngineExecutionContext::new(engine_mode));
            }
        }

        // Register the engine execution context for dependency injection use.
        if let Some(engine_execution_context) = engine_execution_context.as_ref() {
            let engine_execution_context = engine_execution_context.clone();

            dependency_container_builder.register(move |_dependency_container: &DependencyContainer| Ok(engine_execution_context.clone()));
        }

        let squalr_engine = SqualrEngine {
            engine_privileged_state,
            engine_execution_context,
        };

        Ok(squalr_engine)
    }

    pub fn initialize(&self) {
        log::info!("Squalr started");
        log::info!(
            "CPU vector size for accelerated scans: {:?} bytes ({:?} bits), architecture: {}",
            Vectors::get_hardware_vector_size(),
            Vectors::get_hardware_vector_size() * 8,
            Vectors::get_hardware_vector_name(),
        );

        // Initialize privileged engine capabilities if we own them.
        if let Some(engine_privileged_state) = &self.engine_privileged_state {
            engine_privileged_state.initialize(&self.engine_privileged_state);
        }

        // Initialize unprivileged engine capabilities if we own them.
        if let Some(engine_execution_context) = &self.engine_execution_context {
            engine_execution_context.initialize(&self.engine_privileged_state);
        }
    }

    pub fn get_engine_execution_context(&self) -> &Option<Arc<EngineExecutionContext>> {
        &self.engine_execution_context
    }
}
