use crate::commands::{
    project::project_command::ProjectCommand, project_items::project_items_command::ProjectItemsCommand,
    project_symbols::project_symbols_command::ProjectSymbolsCommand,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum UnprivilegedCommand {
    #[structopt(alias = "proj", alias = "p")]
    Project(ProjectCommand),

    #[structopt(alias = "proj_items", alias = "project_items", alias = "pi")]
    ProjectItems(ProjectItemsCommand),

    #[structopt(alias = "proj_symbols", alias = "project_symbols", alias = "ps")]
    ProjectSymbols(ProjectSymbolsCommand),
}
