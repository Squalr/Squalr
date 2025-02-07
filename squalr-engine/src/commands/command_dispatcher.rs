use crate::command_handlers::command_handler::CommandHandler;
use crate::commands::engine_command::EngineCommand;
use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_unprivileged_host::InterProcessUnprivilegedHost;

pub struct CommandDispatcher {
    dispatcher_type: DispatcherType,
}

impl CommandDispatcher {
    pub fn new(dispatcher_type: DispatcherType) -> Self {
        Self { dispatcher_type }
    }

    pub fn dispatch_command(
        &self,
        command: EngineCommand,
    ) {
        match self.dispatcher_type {
            DispatcherType::Standalone => CommandHandler::handle_command(command),
            DispatcherType::InterProcess => InterProcessUnprivilegedHost::get_instance().dispatch_command(command),
        }
    }
}
