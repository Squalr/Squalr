use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterprocessEgress<ResponseType, EventType> {
    EngineResponse(ResponseType),
    EngineEvent(EventType),
}

pub trait TypedEngineResponse<ResponseType>: Sized {
    fn to_engine_response(&self) -> ResponseType;
    fn from_engine_response(response: ResponseType) -> Result<Self, ResponseType>;
}
