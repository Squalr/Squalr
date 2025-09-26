use crate::app_provisioner::updater::app_updater::AppUpdater;
use crate::engine_bindings::standalone::standalone_engine_api_unprivileged_bindings::StandaloneEngineApiUnprivilegedBindings;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::{
    app_provisioner::progress_tracker::ProgressTracker,
    engine_bindings::interprocess::interprocess_engine_api_unprivileged_bindings::InterprocessEngineApiUnprivilegedBindings,
};
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_architecture::vectors::Vectors;
use std::sync::{Arc, RwLock};

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    // The global instance for all engine state, such as snapshots, scan results, running tasks, etc.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    // Execution context that wraps privileged state behind a publically usable API.
    engine_execution_context: Option<Arc<EngineExecutionContext>>,

    dependency_container: DependencyContainer,
}

impl SqualrEngine {
    pub fn new(engine_mode: EngineMode) -> anyhow::Result<Self> {
        let mut engine_privileged_state = None;
        let mut engine_execution_context = None;

        match engine_mode {
            EngineMode::Standalone => {
                engine_privileged_state = Some(EnginePrivilegedState::new(engine_mode));
            }
            EngineMode::PrivilegedShell => {
                engine_privileged_state = Some(EnginePrivilegedState::new(engine_mode));
            }
            EngineMode::UnprivilegedHost => {}
        }

        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandaloneEngineApiUnprivilegedBindings::new(&engine_privileged_state))),
            EngineMode::PrivilegedShell => unreachable!("Unprivileged execution context should never be created from a privileged shell."),
            EngineMode::UnprivilegedHost => Arc::new(RwLock::new(InterprocessEngineApiUnprivilegedBindings::new())),
        };

        match engine_mode {
            EngineMode::Standalone => {
                engine_execution_context = Some(EngineExecutionContext::new(engine_bindings));
            }
            EngineMode::PrivilegedShell => {}
            EngineMode::UnprivilegedHost => {
                engine_execution_context = Some(EngineExecutionContext::new(engine_bindings));
            }
        }

        let squalr_engine = SqualrEngine {
            engine_privileged_state,
            engine_execution_context,
            dependency_container: DependencyContainer::new(),
        };

        log::info!("Squalr started");
        log::info!(
            "CPU vector size for accelerated scans: {:?} bytes ({:?} bits), architecture: {}",
            Vectors::get_hardware_vector_size(),
            Vectors::get_hardware_vector_size() * 8,
            Vectors::get_hardware_vector_name(),
        );

        Ok(squalr_engine)
    }

    pub fn initialize(&mut self) {
        // Initialize unprivileged engine capabilities if we own them.
        if let Some(engine_execution_context) = &self.engine_execution_context {
            engine_execution_context.initialize();

            // Register the engine execution context for dependency injection use.
            self.dependency_container
                .register::<EngineExecutionContext>(engine_execution_context.clone());
        }

        AppUpdater::run_update(ProgressTracker::new());
    }

    pub fn get_engine_execution_context(&self) -> &Option<Arc<EngineExecutionContext>> {
        &self.engine_execution_context
    }

    pub fn get_engine_privileged_state(&self) -> &Option<Arc<EnginePrivilegedState>> {
        &self.engine_privileged_state
    }

    pub fn get_dependency_container(&mut self) -> &DependencyContainer {
        &self.dependency_container
    }
}
