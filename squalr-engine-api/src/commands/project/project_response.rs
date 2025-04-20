use crate::commands::project::list::project_list_response::ProjectListResponse;
use crate::commands::project::open::project_open_response::ProjectOpenResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectResponse {
    Open { project_open_response: ProjectOpenResponse },
    List { project_list_response: ProjectListResponse },
}
