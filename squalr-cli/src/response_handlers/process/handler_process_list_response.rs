use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_list_response(process_response: ProcessResponse) {
    if let ProcessResponse::List { process_list_response } = process_response {
        let processes = process_list_response.processes;

        if processes.is_empty() {
            log::warn!("No processes found!");
            return;
        }

        for process_info in processes {
            log::info!("process_id: {}, name: {}", process_info.get_process_id_raw(), process_info.get_name());
        }
    }
}
