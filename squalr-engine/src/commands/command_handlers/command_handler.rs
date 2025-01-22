use crate::commands::command_handlers::inter_process_command_handler::InterProcessCommandHandler;
use crate::commands::command_handlers::standard_command_handler::StandardCommandHandler;
use crate::commands::engine_command::EngineCommand;

pub trait CommandHandler {
    fn handle_command(
        &self,
        command: &mut EngineCommand,
    );
}

pub enum CommandHandlerType {
    Standard(StandardCommandHandler),
    InterProcess(InterProcessCommandHandler),
}

impl CommandHandler for CommandHandlerType {
    fn handle_command(
        &self,
        command: &mut EngineCommand,
    ) {
        match self {
            Self::Standard(dispatcher) => dispatcher.handle_command(command),
            Self::InterProcess(dispatcher) => dispatcher.handle_command(command),
        }
    }
}
