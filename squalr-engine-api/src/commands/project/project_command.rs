use crate::commands::project::create::project_create_request::ProjectCreateRequest;
use crate::commands::project::export::project_export_request::ProjectExportRequest;
use crate::commands::project::list::project_list_request::ProjectListRequest;
use crate::commands::project::open::project_open_request::ProjectOpenRequest;
use crate::commands::project::rename::project_rename_request::ProjectRenameRequest;
use crate::commands::project::save::project_save_request::ProjectSaveRequest;
use crate::commands::project::{close::project_close_request::ProjectCloseRequest, delete::project_delete_request::ProjectDeleteRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// Create a project.
    Create { project_create_request: ProjectCreateRequest },
    /// Delete a project.
    Delete { project_delete_request: ProjectDeleteRequest },
    /// Open a project.
    Open { project_open_request: ProjectOpenRequest },
    /// Close a project.
    Close { project_close_request: ProjectCloseRequest },
    /// Rename a project.
    Rename { project_rename_request: ProjectRenameRequest },
    /// Save a project.
    Save { project_save_request: ProjectSaveRequest },
    /// Export a project.
    Export { project_export_request: ProjectExportRequest },
    /// List all projects.
    List { project_list_request: ProjectListRequest },
}
