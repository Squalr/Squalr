use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_close_response(process_response: ProcessResponse) {
    if let ProcessResponse::Close { process_close_response } = process_response {
        let process_info = process_close_response.process_info;

        if let Some(process_info) = process_info {
            log::info!("Closed process_id: {}, Name: {}", process_info.get_process_id_raw(), process_info.get_name());
        } else {
            log::info!("Failed to close process");
        }
    }
}
