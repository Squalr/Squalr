use crate::{
    commands::{unprivileged_command::UnprivilegedCommand, unprivileged_command_response::TypedUnprivilegedCommandResponse},
    engine::{engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_execution_context::EngineExecutionContext},
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub trait UnprivilegedCommandRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn to_engine_command(&self) -> UnprivilegedCommand;

    fn send<F>(
        &self,
        execution_context: &Arc<impl EngineExecutionContext + 'static>,
        callback: F,
    ) -> bool
    where
        F: FnOnce(<Self as UnprivilegedCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as UnprivilegedCommandRequest>::ResponseType: TypedUnprivilegedCommandResponse,
    {
        let command = self.to_engine_command();
        let execution_context_dyn: Arc<dyn EngineExecutionContext> = execution_context.clone();

        execution_context.dispatch_unprivileged_command(
            command,
            &execution_context_dyn,
            Box::new(
                move |engine_response| match <Self as UnprivilegedCommandRequest>::ResponseType::from_engine_response(engine_response) {
                    Ok(response) => {
                        callback(response);
                    }
                    Err(unexpected_response) => {
                        log::error!("Received unexpected response variant: {:?}", unexpected_response);
                    }
                },
            ),
        )
    }

    fn send_unprivileged<F>(
        &self,
        engine_bindings: &dyn EngineApiUnprivilegedBindings,
        execution_context: &Arc<impl EngineExecutionContext + 'static>,
        callback: F,
    ) where
        F: FnOnce(<Self as UnprivilegedCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as UnprivilegedCommandRequest>::ResponseType: TypedUnprivilegedCommandResponse,
    {
        let command = self.to_engine_command();

        let execution_context: Arc<dyn EngineExecutionContext> = execution_context.clone();

        if let Err(error) = engine_bindings.dispatch_unprivileged_command(
            command,
            &execution_context,
            Box::new(
                move |engine_response| match <Self as UnprivilegedCommandRequest>::ResponseType::from_engine_response(engine_response) {
                    Ok(response) => {
                        callback(response);
                    }
                    Err(unexpected_response) => {
                        log::error!("Received unexpected response variant: {:?}", unexpected_response);
                    }
                },
            ),
        ) {
            log::error!("Error dispatching command: {}", error);
        }
    }
}
