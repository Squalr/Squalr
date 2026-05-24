use serde::{Deserialize, Serialize};
use squalr_engine_api::{commands::privileged_command_result::PrivilegedCommandResult, engine::engine_event_envelope::EngineEventEnvelope};

/// Defines data that is sent from the engine to the host GUI or CLI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEgress {
    PrivilegedCommandResponse(PrivilegedCommandResult),
    EngineEvent(EngineEventEnvelope),
}
