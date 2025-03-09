use serde::{Deserialize, Serialize};
use squalr_engine_api::{commands::engine_response::EngineResponse, events::engine_event::EngineEvent};

/// Defines data that is sent from the engine to the host GUI or CLI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEgress {
    EngineResponse(EngineResponse),
    EngineEvent(EngineEvent),
}
