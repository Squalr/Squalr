use crate::engine_execution_context::EngineExecutionContext;
use crate::engine_privileged_state::EnginePrivilegedState;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_api::commands::engine_request::EngineRequest;
use squalr_engine_api::commands::engine_response::TypedEngineResponse;
use std::sync::Arc;

pub trait EngineRequestExecutor: EngineRequest + Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineRequestExecutor>::ResponseType;

    fn send<F>(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
        callback: F,
    ) where
        F: FnOnce(<Self as EngineRequestExecutor>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as EngineRequestExecutor>::ResponseType: TypedEngineResponse,
    {
        let command = self.to_engine_command();

        execution_context.dispatch_command(command, move |engine_response| {
            if let Ok(response) = <Self as EngineRequestExecutor>::ResponseType::from_engine_response(engine_response) {
                callback(response);
            }
        });
    }
}
