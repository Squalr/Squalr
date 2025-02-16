use crate::command_dispatchers::command_dispatcher::CommandDispatcher;
use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::process::process_response::ProcessResponse;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ProcessRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType: TypedEngineResponse + Into<ProcessResponse>;

    fn execute(&self) -> Self::ResponseType;

    fn to_engine_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        let command = self.clone().to_engine_command();

        CommandDispatcher::dispatch_command(command, move |engine_response| {
            if let EngineResponse::Process(process_response) = engine_response {
                if let Ok(response) = Self::ResponseType::from_engine_response(EngineResponse::Process(process_response)) {
                    callback(response);
                }
            }
        });
    }
}
