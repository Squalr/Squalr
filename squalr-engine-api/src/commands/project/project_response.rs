use crate::commands::project::list::project_list_response::ProjectListResponse;
use crate::commands::project::open::project_open_response::ProjectOpenResponse;
use crate::commands::project::rename::project_rename_response::ProjectRenameResponse;
use crate::commands::project::save::project_save_response::ProjectSaveResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectResponse {
    Open { project_open_response: ProjectOpenResponse },
    Rename { project_rename_response: ProjectRenameResponse },
    Save { project_save_response: ProjectSaveResponse },
    List { project_list_response: ProjectListResponse },
}
