use crate::commands::{
    project::project_command::ProjectCommand, project_items::project_items_command::ProjectItemsCommand,
    project_symbols::project_symbols_command::ProjectSymbolsCommand,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnprivilegedCommand {
    Project(ProjectCommand),
    ProjectItems(ProjectItemsCommand),
    ProjectSymbols(ProjectSymbolsCommand),
}
