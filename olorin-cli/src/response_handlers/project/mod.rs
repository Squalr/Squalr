pub mod handler_project_list_response;

use crate::response_handlers::project::handler_project_list_response::handle_project_list_response;
use olorin_engine_api::commands::project::project_response::ProjectResponse;

pub fn handle_project_response(cmd: ProjectResponse) {
    match cmd {
        ProjectResponse::List { project_list_response } => handle_project_list_response(project_list_response),
    }
}
