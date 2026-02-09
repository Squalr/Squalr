use crate::{engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, structures::projects::project_manager::ProjectManager};
use std::sync::{Arc, RwLock};

/// Abstraction for unprivileged session state required by command dispatch/execution paths.
pub trait EngineExecutionContext: Send + Sync {
    /// Gets the engine bindings used to dispatch privileged and unprivileged commands.
    fn get_bindings(&self) -> &Arc<RwLock<dyn EngineApiUnprivilegedBindings>>;

    /// Gets the project manager owned by the interactive unprivileged session.
    fn get_project_manager(&self) -> &Arc<ProjectManager>;
}
