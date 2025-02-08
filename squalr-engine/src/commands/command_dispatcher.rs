use crate::commands::dispatched_command::DispatchedCommand;
use crate::commands::engine_command::EngineCommand;
use crate::inter_process::dispatcher_type::DispatcherType;
use uuid::Uuid;

pub struct CommandDispatcher {
    dispatcher_type: DispatcherType,
}

impl CommandDispatcher {
    pub fn new(dispatcher_type: DispatcherType) -> Self {
        Self { dispatcher_type }
    }

    pub fn prepare_dispatch(
        &self,
        command: EngineCommand,
    ) -> DispatchedCommand {
        DispatchedCommand::new(Uuid::new_v4(), command, self.dispatcher_type)
    }
}
