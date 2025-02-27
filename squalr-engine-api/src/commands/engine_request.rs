use crate::commands::engine_command::EngineCommand;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait EngineRequest: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn to_engine_command(&self) -> EngineCommand;
}
