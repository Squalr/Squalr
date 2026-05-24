use squalr_engine_api::commands::process::process_response::ProcessResponse;

pub fn handle_process_icon_response(process_response: ProcessResponse) {
    if let ProcessResponse::Icon { process_icon_response } = process_response {
        for process_icon_entry in process_icon_response.process_icons {
            let icon_summary = process_icon_entry
                .process_icon
                .as_ref()
                .map(|process_icon| format!("{}x{}", process_icon.get_width(), process_icon.get_height()))
                .unwrap_or_else(|| String::from("none"));

            log::info!("process_id: {}, icon_dimensions: {}", process_icon_entry.process_id, icon_summary);
        }
    }
}
