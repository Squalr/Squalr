use crate::{engine_bindings::executable_command_privileged::ExecutableCommandPrivileged, engine_privileged_state::EnginePrivilegedState};
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use std::sync::Arc;

pub trait PrivilegedCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Self::ResponseType;
}

impl ExecutableCommandPrivileged for PrivilegedCommand {
    fn execute(
        &self,
        state: &Arc<EnginePrivilegedState>,
    ) -> PrivilegedCommandResponse {
        self.execute_for_attachment(state, None)
    }

    fn execute_for_attachment(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
        expected_generation: Option<u64>,
    ) -> PrivilegedCommandResponse {
        // Process transitions take the exclusive attachment lock inside the process manager.
        let _attachment = if matches!(
            self,
            PrivilegedCommand::Process(_) | PrivilegedCommand::Settings(_) | PrivilegedCommand::Registry(_) | PrivilegedCommand::TrackableTasks(_)
        ) {
            None
        } else {
            match engine_privileged_state.get_process_manager().attachment_guard() {
                Ok(guard) => {
                    if expected_generation.is_some_and(|generation| generation != engine_privileged_state.get_process_manager().generation()) {
                        return PrivilegedCommandResponse::Failed {
                            message: "Target changed before the command started.".into(),
                        };
                    }
                    Some(guard)
                }
                Err(error) => return PrivilegedCommandResponse::Failed { message: error.to_string() },
            }
        };
        match self {
            PrivilegedCommand::Debugger(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Memory(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Patches(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Plugins(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Process(command) => {
                super::process::process_command_executor::execute_process_command(command, engine_privileged_state, expected_generation)
            }
            PrivilegedCommand::Registry(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Results(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Scan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::PointerScan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::StructScan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Settings(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::TrackableTasks(command) => command.execute(engine_privileged_state),
        }
    }
}
