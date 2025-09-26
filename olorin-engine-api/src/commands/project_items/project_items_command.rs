use crate::commands::project_items::{
    activate::project_items_activate_request::ProjectItemsActivateRequest, list::project_items_list_request::ProjectItemsListRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectItemsCommand {
    /// Activates project items.
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: ProjectItemsActivateRequest,
    },
    /// Lists opened project items.
    List {
        #[structopt(flatten)]
        project_items_list_request: ProjectItemsListRequest,
    },
}
