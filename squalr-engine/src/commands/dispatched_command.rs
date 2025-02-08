use crate::command_handlers::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use uuid::Uuid;

pub struct DispatchedCommand {
    id: Uuid,
    command: EngineCommand,
    dispatcher_type: DispatcherType,
}

impl DispatchedCommand {
    pub fn new(
        id: Uuid,
        command: EngineCommand,
        dispatcher_type: DispatcherType,
    ) -> Self {
        Self { id, command, dispatcher_type }
    }

    pub fn execute(self) {
        let uuid = self.get_id();

        match self.dispatcher_type {
            DispatcherType::Standalone => CommandHandler::handle_command(self.command, uuid),
            DispatcherType::InterProcess => InterProcessUnprivilegedHost::get_instance().dispatch_command(self.command, uuid),
            DispatcherType::None => panic!("Command should not be dispatched from a privileged shell."),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}
