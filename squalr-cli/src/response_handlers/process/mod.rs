pub mod process_response_close;
pub mod process_response_list;
pub mod process_response_open;

use process_response_close::handle_process_response_close;
use process_response_list::handle_process_response_list;
use process_response_open::handle_process_response_open;
use squalr_engine::responses::process::process_response::ProcessResponse;

pub fn handle_process_response(response: ProcessResponse) {
    match response {
        ProcessResponse::List { .. } => handle_process_response_list(response),
        ProcessResponse::Close { .. } => handle_process_response_close(response),
        ProcessResponse::Open { .. } => handle_process_response_open(response),
    }
}
