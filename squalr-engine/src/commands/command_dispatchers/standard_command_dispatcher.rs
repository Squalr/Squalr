use crate::commands::command_handlers::standard_command_handler::StandardCommandHandler;
use crate::commands::engine_command::EngineCommand;

pub struct StandardCommandDispatcher {
    command_handler: StandardCommandHandler,
}

impl StandardCommandDispatcher {
    pub fn new() -> StandardCommandDispatcher {
        Self {
            command_handler: StandardCommandHandler::new(),
        }
    }

    pub fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        // The standard dispatcher just immediately handles the command.
        self.command_handler.handle_command(command);
    }
}
