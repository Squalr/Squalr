use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_open_response(process_response: ProcessResponse) {
    if let ProcessResponse::Open { process_open_response } = process_response {
        let process_info = process_open_response.opened_process_info;

        if let Some(process_info) = process_info {
            log::info!("Opened process_id: {}, Name: {}", process_info.process_id, process_info.name);
        } else {
            log::error!("Failed to open process");
        }
    }
}
