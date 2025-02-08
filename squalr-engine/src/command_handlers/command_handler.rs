use uuid::Uuid;

use crate::command_handlers::memory;
use crate::command_handlers::process;
use crate::command_handlers::project;
use crate::command_handlers::results;
use crate::command_handlers::scan;
use crate::command_handlers::settings;
use crate::commands::engine_command::EngineCommand;

pub enum CommandHandlerType {
    Standalone(),
    InterProcess(),
}

pub struct CommandHandler {}

impl CommandHandler {
    pub fn handle_command(
        command: EngineCommand,
        uuid: Uuid,
    ) {
        match command {
            EngineCommand::Memory(cmd) => memory::handle_memory_command(cmd, uuid),
            EngineCommand::Process(cmd) => process::handle_process_command(cmd, uuid),
            EngineCommand::Project(cmd) => project::handle_project_command(cmd, uuid),
            EngineCommand::Results(cmd) => results::handle_results_command(cmd, uuid),
            EngineCommand::Scan(cmd) => scan::handle_scan_command(cmd, uuid),
            EngineCommand::Settings(cmd) => settings::handle_settings_command(cmd, uuid),
        }
    }
}
