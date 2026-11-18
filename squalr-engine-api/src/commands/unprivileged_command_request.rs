use crate::{
    commands::{
        unprivileged_command::UnprivilegedCommand,
        unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
    },
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
        self.send_with_result(execution_context, move |result| match result {
            Ok(response) => callback(response),
            Err(response) => log::error!("Unprivileged command failed or returned an unexpected response: {:?}", response),
        })
    }

    /// Dispatches a session command with an observable completion after acceptance.
    fn send_with_result<F>(
        &self,
        execution_context: &Arc<impl EngineExecutionContext + 'static>,
        callback: F,
    ) -> bool
    where
        F: FnOnce(Result<Self::ResponseType, UnprivilegedCommandResponse>) + Send + Sync + 'static,
        Self::ResponseType: TypedUnprivilegedCommandResponse,
    {
        let context: Arc<dyn EngineExecutionContext> = execution_context.clone();
        execution_context.dispatch_unprivileged_command(
            self.to_engine_command(),
            &context,
            Box::new(move |response| callback(Self::ResponseType::from_engine_response(response))),
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
