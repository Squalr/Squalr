use crate::app_provisioner::updater::app_updater::AppUpdater;
use crate::engine_bindings::standalone::standalone_engine_api_unprivileged_bindings::StandaloneEngineApiUnprivilegedBindings;
use crate::engine_mode::EngineMode;
use crate::engine_privileged_state::{EnginePrivilegedState, create_engine_privileged_state};
use crate::vectors::Vectors;
use crate::{
    app_provisioner::progress_tracker::ProgressTracker,
    engine_bindings::interprocess::interprocess_engine_api_unprivileged_bindings::InterprocessEngineApiUnprivilegedBindings,
};
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::{Arc, RwLock};

/// Orchestrates commands and responses to and from the engine.
pub struct SqualrEngine {
    /// The global instance for all engine state, such as snapshots, scan results, running tasks, etc.
    engine_privileged_state: Option<Arc<EnginePrivilegedState>>,

    /// Execution context that abstracts privileged state behind a publically usable API.
    engine_unprivileged_state: Option<Arc<EngineUnprivilegedState>>,

    /// Dependency injection manager.
    dependency_container: DependencyContainer,
}

impl SqualrEngine {
    pub fn new(engine_mode: EngineMode) -> anyhow::Result<Self> {
        let mut engine_privileged_state = None;
        let mut engine_unprivileged_state = None;

        match engine_mode {
            EngineMode::Standalone => {
                engine_privileged_state = Some(create_engine_privileged_state(engine_mode)?);
            }
            EngineMode::PrivilegedShell => {
                engine_privileged_state = Some(create_engine_privileged_state(engine_mode)?);
            }
            EngineMode::UnprivilegedHost => {}
        }

        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = match engine_mode {
            EngineMode::Standalone => Arc::new(RwLock::new(StandaloneEngineApiUnprivilegedBindings::new(
                engine_privileged_state
                    .as_ref()
                    .expect("Standalone mode must always initialize privileged state before creating bindings."),
            ))),
            EngineMode::PrivilegedShell => unreachable!("Unprivileged execution context should never be created from a privileged shell."),
            EngineMode::UnprivilegedHost => Arc::new(RwLock::new(InterprocessEngineApiUnprivilegedBindings::new()?)),
        };

        match engine_mode {
            EngineMode::Standalone | EngineMode::UnprivilegedHost => {
                engine_unprivileged_state = Some(EngineUnprivilegedState::new(engine_bindings));
            }
            EngineMode::PrivilegedShell => {}
        }

        let squalr_engine = SqualrEngine {
            engine_privileged_state,
            engine_unprivileged_state,
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
        if let Some(engine_unprivileged_state) = &self.engine_unprivileged_state {
            engine_unprivileged_state.initialize();
        }

        AppUpdater::run_update(ProgressTracker::new());
    }

    /// Gets the engine execution context to allow for API access to the engine privileged state.
    pub fn get_engine_unprivileged_state(&self) -> &Option<Arc<EngineUnprivilegedState>> {
        &self.engine_unprivileged_state
    }

    /// Gets the privileged state for this session. May not be present for non-standalone builds.
    pub fn get_engine_privileged_state(&self) -> &Option<Arc<EnginePrivilegedState>> {
        &self.engine_privileged_state
    }

    /// Gets the dependency injection manager.
    pub fn get_dependency_container(&mut self) -> &DependencyContainer {
        &self.dependency_container
    }
}
