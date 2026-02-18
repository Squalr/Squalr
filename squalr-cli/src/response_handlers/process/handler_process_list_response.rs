use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_list_response(process_response: ProcessResponse) {
    if let ProcessResponse::List { process_list_response } = process_response {
        let processes = process_list_response.processes;

        if processes.is_empty() {
            log::warn!("No processes found!");
            return;
        }

        for process_info in processes {
            let icon_dimensions = process_info
                .get_icon()
                .as_ref()
                .map(|process_icon| format!("{}x{}", process_icon.get_width(), process_icon.get_height()))
                .unwrap_or_else(|| "none".to_string());
            let has_icon = process_info.get_icon().is_some();

            log::info!(
                "process_id: {}, name: {}, is_windowed: {}, has_icon: {}, icon_dimensions: {}",
                process_info.get_process_id_raw(),
                process_info.get_name(),
                process_info.get_is_windowed(),
                has_icon,
                icon_dimensions
            );
        }
    }
}
