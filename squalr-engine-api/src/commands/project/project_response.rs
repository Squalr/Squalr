use crate::commands::project::close::project_close_response::ProjectCloseResponse;
use crate::commands::project::create::project_create_response::ProjectCreateResponse;
use crate::commands::project::export::project_export_response::ProjectExportResponse;
use crate::commands::project::list::project_list_response::ProjectListResponse;
use crate::commands::project::open::project_open_response::ProjectOpenResponse;
use crate::commands::project::rename::project_rename_response::ProjectRenameResponse;
use crate::commands::project::save::project_save_response::ProjectSaveResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectResponse {
    Create { project_create_response: ProjectCreateResponse },
    Open { project_open_response: ProjectOpenResponse },
    Close { project_close_response: ProjectCloseResponse },
    Rename { project_rename_response: ProjectRenameResponse },
    Save { project_save_response: ProjectSaveResponse },
    Export { project_export_response: ProjectExportResponse },
    List { project_list_response: ProjectListResponse },
}
