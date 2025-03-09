use serde::{Deserialize, Serialize};
use squalr_engine_api::{commands::engine_response::EngineResponse, events::engine_event::EngineEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEgress {
    EngineResponse(EngineResponse),
    EngineEvent(EngineEvent),
}
