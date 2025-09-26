pub mod handler_process_close_response;
pub mod handler_process_list_response;
pub mod handler_process_open_response;

use crate::response_handlers::process::handler_process_close_response::handle_process_close_response;
use crate::response_handlers::process::handler_process_list_response::handle_process_list_response;
use crate::response_handlers::process::handler_process_open_response::handle_process_open_response;
use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_response(response: ProcessResponse) {
    match response {
        ProcessResponse::List { .. } => handle_process_list_response(response),
        ProcessResponse::Close { .. } => handle_process_close_response(response),
        ProcessResponse::Open { .. } => handle_process_open_response(response),
    }
}
