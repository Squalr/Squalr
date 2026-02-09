mod memory;
mod pointer_scan;
mod process;
mod project;
mod scan;
mod scan_results;
mod settings;
mod struct_scan;

use crate::response_handlers::memory::handle_memory_response;
use crate::response_handlers::pointer_scan::handle_pointer_scan_response;
use crate::response_handlers::process::handle_process_response;
use crate::response_handlers::project::handle_project_response;
use crate::response_handlers::scan::handle_scan_response;
use crate::response_handlers::scan_results::handle_scan_results_response;
use crate::response_handlers::settings::handle_settings_response;
use crate::response_handlers::struct_scan::handle_struct_scan_response;
use squalr_engine_api::commands::privileged_command_response::PrivilegedCommandResponse;

pub fn handle_engine_response(response: PrivilegedCommandResponse) {
    match response {
        PrivilegedCommandResponse::Scan(response) => handle_scan_response(response),
        PrivilegedCommandResponse::Memory(response) => handle_memory_response(response),
        PrivilegedCommandResponse::Process(response) => handle_process_response(response),
        PrivilegedCommandResponse::Results(response) => handle_scan_results_response(response),
        PrivilegedCommandResponse::Project(response) => handle_project_response(response),
        PrivilegedCommandResponse::PointerScan(response) => handle_pointer_scan_response(response),
        PrivilegedCommandResponse::StructScan(response) => handle_struct_scan_response(response),
        PrivilegedCommandResponse::Settings(response) => handle_settings_response(response),
        PrivilegedCommandResponse::ProjectItems(response) => {
            log::debug!("Unhandled project items response: {:?}", response);
        }
        PrivilegedCommandResponse::TrackableTasks(response) => {
            log::debug!("Unhandled trackable tasks response: {:?}", response);
        }
    }
}
