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
    pub fn handle_command(command: EngineCommand) {
        match command {
            EngineCommand::Memory(cmd) => memory::handle_memory_command(cmd),
            EngineCommand::Process(cmd) => process::handle_process_command(cmd),
            EngineCommand::Project(cmd) => project::handle_project_command(cmd),
            EngineCommand::Results(cmd) => results::handle_results_command(cmd),
            EngineCommand::Scan(cmd) => scan::handle_scan_command(cmd),
            EngineCommand::Settings(cmd) => settings::handle_settings_command(cmd),
        }
    }
}
