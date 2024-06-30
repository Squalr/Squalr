use structopt::StructOpt;
use crate::command_handlers::memory::MemoryCommand;
use crate::command_handlers::process::ProcessCommand;
use crate::command_handlers::project::ProjectCommand;
use crate::command_handlers::scan::ScanCommand;

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(flatten)]
    Memory(MemoryCommand),

    #[structopt(flatten)]
    Process(ProcessCommand),

    #[structopt(flatten)]
    Project(ProjectCommand),

    #[structopt(flatten)]
    Scan(ScanCommand),
}
