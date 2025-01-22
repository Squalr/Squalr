use crate::commands::command_dispatchers::inter_process_command_dispatcher::InterProcessCommandDispatcher;
use crate::commands::command_dispatchers::standard_command_dispatcher::StandardCommandDispatcher;
use crate::commands::engine_command::EngineCommand;

pub trait CommandDispatcher {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    );
}

pub enum CommandDispatcherType {
    Standard(StandardCommandDispatcher),
    InterProcess(InterProcessCommandDispatcher),
}

impl CommandDispatcher for CommandDispatcherType {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        match self {
            Self::Standard(dispatcher) => dispatcher.dispatch_command(command),
            Self::InterProcess(dispatcher) => dispatcher.dispatch_command(command),
        }
    }
}
