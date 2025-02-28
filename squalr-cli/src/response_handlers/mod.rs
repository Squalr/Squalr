mod memory;
mod process;
mod project;
mod scan;
mod scan_results;
mod settings;

use crate::response_handlers::memory::handle_memory_response;
use crate::response_handlers::process::handle_process_response;
use crate::response_handlers::project::handle_project_response;
use crate::response_handlers::scan::handle_scan_response;
use crate::response_handlers::scan_results::handle_scan_results_response;
use crate::response_handlers::settings::handle_settings_response;
use squalr_engine_api::commands::engine_response::EngineResponse;

pub fn handle_engine_response(response: EngineResponse) {
    match response {
        EngineResponse::Scan(response) => handle_scan_response(response),
        EngineResponse::Memory(response) => handle_memory_response(response),
        EngineResponse::Process(response) => handle_process_response(response),
        EngineResponse::Results(response) => handle_scan_results_response(response),
        EngineResponse::Project(response) => handle_project_response(response),
        EngineResponse::Scan(response) => handle_scan_response(response),
        EngineResponse::Settings(response) => handle_settings_response(response),
    }
}
