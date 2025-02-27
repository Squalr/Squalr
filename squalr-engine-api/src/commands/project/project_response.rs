use crate::commands::project::list::project_list_response::ProjectListResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectResponse {
    List { project_list_response: ProjectListResponse },
}
