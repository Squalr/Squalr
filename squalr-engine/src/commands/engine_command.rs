use crate::commands::command_handler::CommandHandler;
use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::results::results_command::ResultsCommand;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::settings::settings_command::SettingsCommand;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum EngineCommand {
    #[structopt(alias = "mem", alias = "m")]
    Memory(MemoryCommand),

    #[structopt(alias = "proc", alias = "pr")]
    Process(ProcessCommand),

    #[structopt(alias = "proj", alias = "p")]
    Project(ProjectCommand),

    #[structopt(alias = "res", alias = "r")]
    Results(ResultsCommand),

    #[structopt(alias = "scan", alias = "s")]
    Scan(ScanCommand),

    #[structopt(alias = "set", alias = "st")]
    Settings(SettingsCommand),
}

impl CommandHandler for EngineCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            EngineCommand::Memory(command) => {
                command.handle(uuid);
            }
            EngineCommand::Process(command) => {
                command.handle(uuid);
            }
            EngineCommand::Project(command) => {
                command.handle(uuid);
            }
            EngineCommand::Results(command) => {
                command.handle(uuid);
            }
            EngineCommand::Scan(command) => {
                command.handle(uuid);
            }
            EngineCommand::Settings(command) => {
                command.handle(uuid);
            }
        }
    }
}
