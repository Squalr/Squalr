use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::results::results_response::ResultsResponse;
use crate::squalr_engine::SqualrEngine;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ResultsRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType: TypedEngineResponse + Into<ResultsResponse>;

    fn execute(&self) -> Self::ResponseType;

    fn to_engine_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        let command = self.clone().to_engine_command();

        SqualrEngine::dispatch_command(command, move |engine_response| {
            if let EngineResponse::Results(process_response) = engine_response {
                if let Ok(response) = Self::ResponseType::from_engine_response(EngineResponse::Results(process_response)) {
                    callback(response);
                }
            }
        });
    }
}
