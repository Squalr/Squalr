use std::sync::Arc;

use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::TypedEngineResponse;
use crate::engine_execution_context::EngineExecutionContext;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait EngineRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType;

    fn to_engine_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
        callback: F,
    ) where
        F: Fn(Self::ResponseType) + Clone + Send + Sync + 'static,
        Self::ResponseType: TypedEngineResponse,
    {
        let command = self.clone().to_engine_command();

        execution_context.dispatch_command(command, move |engine_response| {
            if let Ok(response) = Self::ResponseType::from_engine_response(engine_response) {
                callback(response);
            }
        });
    }
}
