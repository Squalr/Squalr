use crate::commands::command_dispatchers::inter_process_command_dispatcher::InterProcessCommandDispatcher;
use crate::commands::command_handlers::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;

pub trait CommandDispatcher {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    );
}

pub enum CommandDispatcherType {
    Standard(),
    InterProcess(InterProcessCommandDispatcher),
}

impl CommandDispatcher for CommandDispatcherType {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        match self {
            // The standard dispatcher just immediately handles the command.
            Self::Standard() => CommandHandler::handle_command(command),
            Self::InterProcess(dispatcher) => dispatcher.dispatch_command(command),
        }
    }
}
