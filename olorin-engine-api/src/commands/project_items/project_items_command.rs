use crate::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectItemsCommand {
    /// Activates project items.
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: ProjectItemsActivateRequest,
    },
}
