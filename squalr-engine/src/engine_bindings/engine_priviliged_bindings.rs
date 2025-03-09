use crate::{engine_execution_context::EngineExecutionContext, engine_privileged_state::EnginePrivilegedState};
use std::sync::Arc;

/// Defines functionality that can be invoked by the engine for the GUI or CLI to handle.
pub trait EnginePrivilegedBindings: Send + Sync {
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String>;
}
