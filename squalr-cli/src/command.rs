use structopt::StructOpt;
use crate::command_handlers::process::ProcessCommand;
use crate::command_handlers::scan::ScanCommand;
use crate::command_handlers::project::ProjectCommand;

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Process related commands
    Process(ProcessCommand),
    /// Scan related commands
    Scan(ScanCommand),
    /// Project related commands
    Project(ProjectCommand),
}
