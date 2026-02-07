pub mod handler_project_list_response;

use crate::response_handlers::project::handler_project_list_response::handle_project_list_response;
use squalr_engine_api::commands::project::project_response::ProjectResponse;

pub fn handle_project_response(cmd: ProjectResponse) {
    match cmd {
        ProjectResponse::List { project_list_response } => handle_project_list_response(project_list_response),
        ProjectResponse::Create { project_create_response } => {
            log::debug!("Unhandled project create response: {:?}", project_create_response);
        }
        ProjectResponse::Delete { project_delete_response } => {
            log::debug!("Unhandled project delete response: {:?}", project_delete_response);
        }
        ProjectResponse::Open { project_open_response } => {
            log::debug!("Unhandled project open response: {:?}", project_open_response);
        }
        ProjectResponse::Close { project_close_response } => {
            log::debug!("Unhandled project close response: {:?}", project_close_response);
        }
        ProjectResponse::Rename { project_rename_response } => {
            log::debug!("Unhandled project rename response: {:?}", project_rename_response);
        }
        ProjectResponse::Save { project_save_response } => {
            log::debug!("Unhandled project save response: {:?}", project_save_response);
        }
        ProjectResponse::Export { project_export_response } => {
            log::debug!("Unhandled project export response: {:?}", project_export_response);
        }
    }
}
