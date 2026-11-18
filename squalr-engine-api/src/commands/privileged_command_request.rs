use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use crate::engine::engine_execution_context::EngineExecutionContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub trait PrivilegedCommandRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn to_engine_command(&self) -> PrivilegedCommand;

    fn send<F>(
        &self,
        engine_execution_context: &Arc<impl EngineExecutionContext + 'static>,
        callback: F,
    ) -> bool
    where
        F: FnOnce(<Self as PrivilegedCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as PrivilegedCommandRequest>::ResponseType: TypedPrivilegedCommandResponse,
    {
        self.send_with_result(engine_execution_context, move |result| match result {
            Ok(response) => callback(response),
            Err(response) => log::error!("Privileged command failed or returned an unexpected response: {:?}", response),
        })
    }

    /// Dispatches a request with an observable terminal result, including transport failures.
    /// A false return indicates rejection before acceptance.
    fn send_with_result<F>(
        &self,
        engine_execution_context: &Arc<impl EngineExecutionContext + 'static>,
        callback: F,
    ) -> bool
    where
        F: FnOnce(Result<Self::ResponseType, PrivilegedCommandResponse>) + Send + Sync + 'static,
        Self::ResponseType: TypedPrivilegedCommandResponse,
    {
        engine_execution_context.dispatch_privileged_command(
            self.to_engine_command(),
            Box::new(move |response| callback(Self::ResponseType::from_engine_response(response))),
        )
    }

    fn send_unprivileged<F>(
        &self,
        engine_bindings: &dyn EngineApiUnprivilegedBindings,
        callback: F,
    ) -> bool
    where
        F: FnOnce(<Self as PrivilegedCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as PrivilegedCommandRequest>::ResponseType: TypedPrivilegedCommandResponse,
    {
        let command = self.to_engine_command();

        if let Err(error) = engine_bindings.dispatch_privileged_command(
            command,
            Box::new(
                move |engine_response| match <Self as PrivilegedCommandRequest>::ResponseType::from_engine_response(engine_response) {
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
            return false;
        }

        true
    }
}
