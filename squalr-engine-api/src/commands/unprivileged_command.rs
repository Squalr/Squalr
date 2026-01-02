use crate::commands::{project::project_command::ProjectCommand, project_items::project_items_command::ProjectItemsCommand};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum UnprivilegedCommand {
    #[structopt(alias = "proj", alias = "p")]
    Project(ProjectCommand),

    #[structopt(alias = "proj_items", alias = "pi")]
    ProjectItems(ProjectItemsCommand),
}
