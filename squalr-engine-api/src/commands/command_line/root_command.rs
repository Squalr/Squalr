use super::command::CommandLineCommand;
use super::memory::CommandLineMemoryCommand;
use super::plugins::CommandLinePluginsCommand;
use super::pointer_scan::CommandLinePointerScanCommand;
use super::process::CommandLineProcessCommand;
use super::project::CommandLineProjectCommand;
use super::project_items::CommandLineProjectItemsCommand;
use super::project_symbols::CommandLineProjectSymbolsCommand;
use super::registry::CommandLineRegistryCommand;
use super::scan::CommandLineScanCommand;
use super::scan_results::CommandLineScanResultsCommand;
use super::settings::CommandLineSettingsCommand;
use super::struct_scan::CommandLineStructScanCommand;
use super::trackable_tasks::CommandLineTrackableTasksCommand;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use structopt::StructOpt;
#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineRootCommand {
    #[structopt(alias = "mem", alias = "m")]
    Memory(CommandLineMemoryCommand),
    #[structopt(alias = "plug", alias = "plugins")]
    Plugins(CommandLinePluginsCommand),
    #[structopt(alias = "proc", alias = "pr")]
    Process(CommandLineProcessCommand),
    #[structopt(alias = "reg")]
    Registry(CommandLineRegistryCommand),
    #[structopt(alias = "res", alias = "r")]
    Results(CommandLineScanResultsCommand),
    #[structopt(alias = "scan", alias = "s")]
    Scan(CommandLineScanCommand),
    #[structopt(alias = "pscan")]
    PointerScan(CommandLinePointerScanCommand),
    #[structopt(alias = "sscan")]
    StructScan(CommandLineStructScanCommand),
    #[structopt(alias = "set", alias = "st")]
    Settings(CommandLineSettingsCommand),
    #[structopt(alias = "tasks", alias = "tt")]
    TrackableTasks(CommandLineTrackableTasksCommand),
    #[structopt(alias = "proj", alias = "p")]
    Project(CommandLineProjectCommand),
    #[structopt(alias = "proj_items", alias = "project_items", alias = "pi")]
    ProjectItems(CommandLineProjectItemsCommand),
    #[structopt(alias = "proj_symbols", alias = "project_symbols", alias = "ps")]
    ProjectSymbols(CommandLineProjectSymbolsCommand),
}

impl From<CommandLineRootCommand> for CommandLineCommand {
    fn from(command: CommandLineRootCommand) -> Self {
        match command {
            CommandLineRootCommand::Memory(command) => Self::Privileged(PrivilegedCommand::Memory(command.into())),
            CommandLineRootCommand::Plugins(command) => Self::Privileged(PrivilegedCommand::Plugins(command.into())),
            CommandLineRootCommand::Process(command) => Self::Privileged(PrivilegedCommand::Process(command.into())),
            CommandLineRootCommand::Registry(command) => Self::Privileged(PrivilegedCommand::Registry(command.into())),
            CommandLineRootCommand::Results(command) => Self::Privileged(PrivilegedCommand::Results(command.into())),
            CommandLineRootCommand::Scan(command) => Self::Privileged(PrivilegedCommand::Scan(command.into())),
            CommandLineRootCommand::PointerScan(command) => Self::Privileged(PrivilegedCommand::PointerScan(command.into())),
            CommandLineRootCommand::StructScan(command) => Self::Privileged(PrivilegedCommand::StructScan(command.into())),
            CommandLineRootCommand::Settings(command) => Self::Privileged(PrivilegedCommand::Settings(command.into())),
            CommandLineRootCommand::TrackableTasks(command) => Self::Privileged(PrivilegedCommand::TrackableTasks(command.into())),
            CommandLineRootCommand::Project(command) => Self::Unprivileged(UnprivilegedCommand::Project(command.into())),
            CommandLineRootCommand::ProjectItems(command) => Self::Unprivileged(UnprivilegedCommand::ProjectItems(command.into())),
            CommandLineRootCommand::ProjectSymbols(command) => Self::Unprivileged(UnprivilegedCommand::ProjectSymbols(command.into())),
        }
    }
}
