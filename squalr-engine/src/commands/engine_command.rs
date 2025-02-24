use std::sync::Arc;

use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::process::process_command::ProcessCommand;
use crate::commands::project::project_command::ProjectCommand;
use crate::commands::results::results_command::ResultsCommand;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::{commands::engine_response::EngineResponse, engine_execution_context::EngineExecutionContext};
use interprocess_shell::interprocess_ingress::ExecutableRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

impl ExecutableRequest<EngineResponse, EngineExecutionContext> for EngineCommand {
    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> EngineResponse {
        match self {
            EngineCommand::Memory(command) => command.execute(execution_context),
            EngineCommand::Process(command) => command.execute(execution_context),
            EngineCommand::Project(command) => command.execute(execution_context),
            EngineCommand::Results(command) => command.execute(execution_context),
            EngineCommand::Scan(command) => command.execute(execution_context),
            EngineCommand::Settings(command) => command.execute(execution_context),
        }
    }
}
