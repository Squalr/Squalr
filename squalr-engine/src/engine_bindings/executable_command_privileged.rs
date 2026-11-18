use crate::engine_privileged_state::EnginePrivilegedState;
use serde::{Deserialize, Serialize};
use squalr_engine_api::commands::privileged_command_result::PrivilegedCommandResult;
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use std::sync::Arc;

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    PrivilegedCommand(PrivilegedCommand),
}

pub trait ExecutableCommandPrivileged {
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> PrivilegedCommandResponse;

    fn execute_for_attachment(
        &self,
        state: &Arc<EnginePrivilegedState>,
        _expected_generation: Option<u64>,
    ) -> PrivilegedCommandResponse {
        self.execute(state)
    }
}

/// Runs a worker command and constructs its transport-independent response envelope.
pub fn execute_privileged_command(
    state: &Arc<EnginePrivilegedState>,
    command: PrivilegedCommand,
    expected_generation: Option<u64>,
) -> PrivilegedCommandResult {
    let response = command.execute_for_attachment(state, expected_generation);
    let catalog = command
        .should_include_privileged_registry_catalog()
        .then(|| state.get_privileged_registry_catalog());
    PrivilegedCommandResult::new(response, catalog)
}
