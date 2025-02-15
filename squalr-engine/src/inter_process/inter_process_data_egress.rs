use crate::{commands::engine_response::EngineResponse, events::engine_event::EngineEvent};
use serde::{Deserialize, Serialize};

/// Represents data that is sent from the engine to the host (GUI/CLI/IPC host).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterProcessDataEgress {
    Event(EngineEvent),
    Response(EngineResponse),
}
