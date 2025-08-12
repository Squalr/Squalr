use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use crate::engine::engine_execution_context::EngineExecutionContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, RwLock};

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
        match execution_context.get_bindings().read() {
            Ok(engine_bindings) => {
                self.send_unprivileged(&*engine_bindings, callback);
            }
            Err(error) => log::error!("Error getting engine execution context bindings: {}", error),
        };
    }

    fn send_unprivileged<F>(
        &self,
        engine_bindings: &dyn EngineApiUnprivilegedBindings,
        callback: F,
    ) where
        F: FnOnce(<Self as EngineCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as EngineCommandRequest>::ResponseType: TypedEngineCommandResponse,
    {
        let command = self.to_engine_command();

        if let Err(error) = engine_bindings.dispatch_command(
            command,
            Box::new(move |engine_response| {
                if let Ok(response) = <Self as EngineCommandRequest>::ResponseType::from_engine_response(engine_response) {
                    callback(response);
                }
            }),
        ) {
            log::error!("Error dispatching command: {}", error);
        }
    }

    fn send_privileged<F>(
        &self,
        engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        callback: F,
    ) where
        F: FnOnce(<Self as EngineCommandRequest>::ResponseType) + Clone + Send + Sync + 'static,
        <Self as EngineCommandRequest>::ResponseType: TypedEngineCommandResponse,
    {
        let command = self.to_engine_command();

        match engine_bindings.read() {
            Ok(engine_bindings) => {
                if let Err(error) = engine_bindings.dispatch_command(
                    command,
                    Box::new(move |engine_response| {
                        if let Ok(response) = <Self as EngineCommandRequest>::ResponseType::from_engine_response(engine_response) {
                            callback(response);
                        }
                    }),
                ) {
                    log::error!("Error dispatching command: {}", error);
                }
            }
            Err(error) => {
                log::error!("Error acquiring engine binding lock to dispatch command: {}", error);
            }
        }
    }
}
