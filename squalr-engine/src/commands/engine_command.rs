use crate::commands::command_handlers::memory::MemoryCommand;
use crate::commands::command_handlers::process::ProcessCommand;
use crate::commands::command_handlers::project::ProjectCommand;
use crate::commands::command_handlers::results::results_command::ResultsCommand;
use crate::commands::command_handlers::scan::ScanCommand;
use crate::commands::command_handlers::settings::SettingsCommand;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
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
