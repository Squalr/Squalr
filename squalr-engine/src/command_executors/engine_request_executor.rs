use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_api::commands::engine_command_request::EngineCommandRequest;
use squalr_engine_api::commands::engine_command_response::TypedEngineCommandResponse;
use std::sync::Arc;

pub trait EngineCommandRequestExecutor: EngineCommandRequest + Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType;

    fn send<F>(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
        callback: F,
    ) where
        F: FnOnce(<Self as EngineCommandRequestExecutor>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as EngineCommandRequestExecutor>::ResponseType: TypedEngineCommandResponse,
    {
        let command = self.to_engine_command();

        execution_context.dispatch_command(command, move |engine_response| {
            if let Ok(response) = <Self as EngineCommandRequestExecutor>::ResponseType::from_engine_response(engine_response) {
                callback(response);
            }
        });
    }
}
