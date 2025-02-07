use crate::commands::engine_command::EngineCommand;
use serde::{Deserialize, Serialize};

/// Represents data that is sent from the host (GUI/CLI/IPC host) to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterProcessDataIngress {
    Command(EngineCommand),
}
