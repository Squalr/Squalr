use crate::commands::project::close::project_close_request::ProjectCloseRequest;
use crate::commands::project::create::project_create_request::ProjectCreateRequest;
use crate::commands::project::list::project_list_request::ProjectListRequest;
use crate::commands::project::open::project_open_request::ProjectOpenRequest;
use crate::commands::project::rename::project_rename_request::ProjectRenameRequest;
use crate::commands::project::save::project_save_request::ProjectSaveRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// Create a project.
    Create {
        #[structopt(flatten)]
        project_create_request: ProjectCreateRequest,
    },
    /// Open a project.
    Open {
        #[structopt(flatten)]
        project_open_request: ProjectOpenRequest,
    },
    /// Close a project.
    Close {
        #[structopt(flatten)]
        project_close_request: ProjectCloseRequest,
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
