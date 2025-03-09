use crate::{engine_execution_context::EngineExecutionContext, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::{engine_command::EngineCommand, engine_response::EngineResponse};
use std::sync::Arc;

/// Defines the functionality that is sent to the engine from a GUI, CLI, etc.
pub trait EngineUnprivilegedBindings: Send + Sync {
    fn initialize(
        &mut self,
        engine_privileged_state: &Option<Arc<EnginePrivilegedState>>,
        engine_execution_context: &Option<Arc<EngineExecutionContext>>,
    ) -> Result<(), String>;

    fn dispatch_command(
        &self,
        command: EngineCommand,
        callback: Box<dyn FnOnce(EngineResponse) + Send + Sync + 'static>,
    ) -> Result<(), String>;
}
