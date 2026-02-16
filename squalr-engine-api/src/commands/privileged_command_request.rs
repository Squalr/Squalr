use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
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
        match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => self.send_unprivileged(&*engine_bindings, callback),
            Err(error) => {
                log::error!("Error getting engine execution context bindings: {}", error);
                false
            }
        }
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
