#![forbid(unsafe_code)]

pub mod mocks;

use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
use std::sync::{Arc, OnceLock, RwLock};

/// Provides a shared execution context for integration tests that require `EngineUnprivilegedState`.
/// This avoids repeated logger initialization noise across test processes.
pub fn shared_execution_context() -> Arc<EngineUnprivilegedState> {
    static SHARED_EXECUTION_CONTEXT: OnceLock<Arc<EngineUnprivilegedState>> = OnceLock::new();

    SHARED_EXECUTION_CONTEXT
        .get_or_init(|| {
            let no_op_bindings = Arc::new(RwLock::new(NoOpEngineBindings {
                privileged_response: MemoryWriteResponse { success: true }.to_engine_response(),
                unprivileged_response: ProjectListResponse::default().to_engine_response(),
            }));
            EngineUnprivilegedState::new(no_op_bindings)
        })
        .clone()
}

struct NoOpEngineBindings {
    privileged_response: PrivilegedCommandResponse,
    unprivileged_response: UnprivilegedCommandResponse,
}

impl EngineApiUnprivilegedBindings for NoOpEngineBindings {
    fn dispatch_privileged_command(
        &self,
        _engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        callback(self.privileged_response.clone());
        Ok(())
    }

    fn dispatch_unprivileged_command(
        &self,
        _engine_command: UnprivilegedCommand,
        _engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), EngineBindingError> {
        callback(self.unprivileged_response.clone());
        Ok(())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
        let (_event_sender, event_receiver) = unbounded();
        Ok(event_receiver)
    }
}
