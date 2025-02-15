use crate::commands::engine_command::EngineCommand;
use crate::responses::engine_response::{EngineResponse, TypedEngineResponse};
use crate::responses::process::process_response::ProcessResponse;
use crate::squalr_engine::SqualrEngine;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ProcessRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType: TypedEngineResponse + Into<ProcessResponse>;

    fn to_command(&self) -> EngineCommand;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        let command = self.clone().to_command();

        SqualrEngine::dispatch_command(command, move |engine_response| {
            if let EngineResponse::Process(process_response) = engine_response {
                if let Ok(response) = Self::ResponseType::from_response(EngineResponse::Process(process_response)) {
                    callback(response);
                }
            }
        });
    }
}
