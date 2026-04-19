use squalr_engine_api::commands::project_items::project_items_response::ProjectItemsResponse;

pub fn handle_project_items_response(project_items_response: ProjectItemsResponse) {
    match project_items_response {
        ProjectItemsResponse::List { project_items_list_response } => {
            let opened_project_name = project_items_list_response
                .opened_project_info
                .as_ref()
                .map(|opened_project_info| opened_project_info.get_name().to_string())
                .unwrap_or_else(|| String::from("(no project)"));
            log::info!(
                "project: {}, project item count: {}",
                opened_project_name,
                project_items_list_response.opened_project_items.len()
            );

            for (project_item_ref, project_item) in project_items_list_response.opened_project_items {
                log::info!(
                    "path: {}, name: {}, type: {}",
                    project_item_ref.get_project_item_path().display(),
                    project_item.get_field_name(),
                    project_item.get_item_type().get_project_item_type_id()
                );
            }
        }
        ProjectItemsResponse::PromoteSymbol {
            project_items_promote_symbol_response,
        } => {
            log::info!(
                "promoted {} symbol(s), reused {} existing symbol(s), conflicts {}: {}",
                project_items_promote_symbol_response.promoted_symbol_count,
                project_items_promote_symbol_response.reused_symbol_count,
                project_items_promote_symbol_response.conflicts.len(),
                project_items_promote_symbol_response
                    .promoted_symbol_keys
                    .join(", ")
            );
        }
        ProjectItemsResponse::ConvertSymbolRef {
            project_items_convert_symbol_ref_response,
        } => {
            log::info!(
                "converted {} symbol-ref project item(s)",
                project_items_convert_symbol_ref_response.converted_project_item_count
            );
        }
        ProjectItemsResponse::Add { project_items_add_response } => {
            log::debug!("Unhandled project items add response: {:?}", project_items_add_response);
        }
        ProjectItemsResponse::Activate {
            project_items_activate_response,
        } => {
            log::debug!("Unhandled project items activate response: {:?}", project_items_activate_response);
        }
        ProjectItemsResponse::Create { project_items_create_response } => {
            log::debug!("Unhandled project items create response: {:?}", project_items_create_response);
        }
        ProjectItemsResponse::Delete { project_items_delete_response } => {
            log::debug!("Unhandled project items delete response: {:?}", project_items_delete_response);
        }
        ProjectItemsResponse::Duplicate {
            project_items_duplicate_response,
        } => {
            log::debug!("Unhandled project items duplicate response: {:?}", project_items_duplicate_response);
        }
        ProjectItemsResponse::Move { project_items_move_response } => {
            log::debug!("Unhandled project items move response: {:?}", project_items_move_response);
        }
        ProjectItemsResponse::Rename { project_items_rename_response } => {
            log::debug!("Unhandled project items rename response: {:?}", project_items_rename_response);
        }
        ProjectItemsResponse::Reorder {
            project_items_reorder_response,
        } => {
            log::debug!("Unhandled project items reorder response: {:?}", project_items_reorder_response);
        }
    }
}
