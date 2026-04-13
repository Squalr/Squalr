use squalr_engine_api::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;

pub fn handle_project_symbols_response(project_symbols_response: ProjectSymbolsResponse) {
    match project_symbols_response {
        ProjectSymbolsResponse::List { project_symbols_list_response } => {
            let opened_project_name = project_symbols_list_response
                .opened_project_info
                .as_ref()
                .map(|opened_project_info| opened_project_info.get_name().to_string())
                .unwrap_or_else(|| String::from("(no project)"));
            let Some(project_symbol_catalog) = project_symbols_list_response.project_symbol_catalog else {
                log::warn!("No opened project symbol catalog is available.");
                return;
            };

            log::info!(
                "project: {}, rooted symbol count: {}, symbol type count: {}",
                opened_project_name,
                project_symbol_catalog.get_rooted_symbols().len(),
                project_symbol_catalog.get_struct_layout_descriptors().len()
            );

            for rooted_symbol in project_symbol_catalog.get_rooted_symbols() {
                log::info!(
                    "symbol: {}, key: {}, type: {}, locator: {}",
                    rooted_symbol.get_display_name(),
                    rooted_symbol.get_symbol_key(),
                    rooted_symbol.get_struct_layout_id(),
                    rooted_symbol.get_root_locator()
                );

                for (metadata_key, metadata_value) in rooted_symbol.get_metadata() {
                    log::info!("  metadata: {}={}", metadata_key, metadata_value);
                }
            }

            for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
                log::info!(
                    "type: {}, fields={}",
                    struct_layout_descriptor.get_struct_layout_id(),
                    struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .len()
                );
            }
        }
        ProjectSymbolsResponse::Create {
            project_symbols_create_response,
        } => {
            log::info!(
                "created rooted symbol: success={}, symbol_key={}",
                project_symbols_create_response.success,
                project_symbols_create_response.created_symbol_key
            );
        }
        ProjectSymbolsResponse::Rename {
            project_symbols_rename_response,
        } => {
            log::info!(
                "renamed rooted symbol: success={}, symbol_key={}",
                project_symbols_rename_response.success,
                project_symbols_rename_response.symbol_key
            );
        }
        ProjectSymbolsResponse::Delete {
            project_symbols_delete_response,
        } => {
            log::info!(
                "deleted rooted symbols: success={}, count={}",
                project_symbols_delete_response.success,
                project_symbols_delete_response.deleted_symbol_count
            );
        }
    }
}
