use crate::commands::project::list::project_list_request::ProjectListRequest;
use crate::commands::project::open::project_open_request::ProjectOpenRequest;
use crate::commands::project::rename::project_rename_request::ProjectRenameRequest;
use crate::commands::project::save::project_save_request::ProjectSaveRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// Open a project.
    Open {
        #[structopt(flatten)]
        project_open_request: ProjectOpenRequest,
    },
    /// Rename a project.
    Rename {
        #[structopt(flatten)]
        project_rename_request: ProjectRenameRequest,
    },
    /// Save a project.
    Save {
        #[structopt(flatten)]
        project_save_request: ProjectSaveRequest,
    },
    /// List all projects.
    List {
        #[structopt(flatten)]
        project_list_request: ProjectListRequest,
    },
}
