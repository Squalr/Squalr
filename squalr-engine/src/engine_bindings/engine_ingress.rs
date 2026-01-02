use serde::{Deserialize, Serialize};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    PrivilegedCommand(PrivilegedCommand),
}
