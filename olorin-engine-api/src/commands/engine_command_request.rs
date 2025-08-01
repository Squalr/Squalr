use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::engine::engine_execution_context::EngineExecutionContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub trait EngineCommandRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn to_engine_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
        callback: F,
    ) where
        F: FnOnce(<Self as EngineCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as EngineCommandRequest>::ResponseType: TypedEngineCommandResponse,
    {
        let command = self.to_engine_command();

        execution_context.dispatch_command(command, move |engine_response| {
            if let Ok(response) = <Self as EngineCommandRequest>::ResponseType::from_engine_response(engine_response) {
                callback(response);
            }
        });
    }
}
