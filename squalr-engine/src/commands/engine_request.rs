use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::TypedEngineResponse;
use crate::squalr_engine::SqualrEngine;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait EngineRequest: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    type ResponseType: TypedEngineResponse;

    fn execute(&self) -> Self::ResponseType;

    fn to_engine_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        let command = self.to_engine_command();

        SqualrEngine::dispatch_command(command, move |engine_response| {
            if let Ok(response) = Self::ResponseType::from_engine_response(engine_response) {
                callback(response);
            }
        });
    }
}
