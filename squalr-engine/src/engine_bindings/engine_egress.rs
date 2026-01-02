use serde::{Deserialize, Serialize};
use squalr_engine_api::{commands::privileged_command_response::PrivilegedCommandResponse, events::engine_event::EngineEvent};

/// Defines data that is sent from the engine to the host GUI or CLI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEgress {
    PrivilegedCommandResponse(PrivilegedCommandResponse),
    EngineEvent(EngineEvent),
}
