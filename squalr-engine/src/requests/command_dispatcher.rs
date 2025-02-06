use crate::command_handlers::command_handler::CommandHandler;
use crate::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;
use crate::requests::engine_command::EngineCommand;

pub trait CommandDispatcher {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    );
}

pub enum CommandDispatcherType {
    Standalone(),
    InterProcess(),
}

impl CommandDispatcher for CommandDispatcherType {
    fn dispatch_command(
        &self,
        command: &mut EngineCommand,
    ) {
        match self {
            Self::Standalone() => CommandHandler::handle_command(command),
            Self::InterProcess() => InterProcessUnprivilegedHost::get_instance().dispatch_command(command),
        }
    }
}
