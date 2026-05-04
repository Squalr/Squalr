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
                "project: {}, module count: {}, symbol field count: {}, symbol type count: {}",
                opened_project_name,
                project_symbol_catalog.get_symbol_modules().len(),
                project_symbol_catalog.get_symbol_claims().len(),
                project_symbol_catalog.get_struct_layout_descriptors().len()
            );

            for symbol_module in project_symbol_catalog.get_symbol_modules() {
                log::info!("module: {}, size: 0x{:X}", symbol_module.get_module_name(), symbol_module.get_size());
            }

            for symbol_claim in project_symbol_catalog.get_symbol_claims() {
                log::info!(
                    "symbol: {}, type: {}, locator: {}",
                    symbol_claim.get_display_name(),
                    symbol_claim.get_struct_layout_id(),
                    symbol_claim.get_locator()
                );

                for (metadata_key, metadata_value) in symbol_claim.get_metadata() {
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
                "created symbol claim: success={}, symbol_locator_key={}",
                project_symbols_create_response.success,
                project_symbols_create_response.created_symbol_locator_key
            );
        }
        ProjectSymbolsResponse::CreateModule {
            project_symbols_create_module_response,
        } => {
            log::info!(
                "created module root: success={}, module={}",
                project_symbols_create_module_response.success,
                project_symbols_create_module_response.module_name
            );
        }
        ProjectSymbolsResponse::Rename {
            project_symbols_rename_response,
        } => {
            log::info!(
                "renamed symbol claim: success={}, symbol_locator_key={}",
                project_symbols_rename_response.success,
                project_symbols_rename_response.symbol_locator_key
            );
        }
        ProjectSymbolsResponse::RenameModule {
            project_symbols_rename_module_response,
        } => {
            log::info!(
                "renamed module root: success={}, module={}",
                project_symbols_rename_module_response.success,
                project_symbols_rename_module_response.module_name
            );
        }
        ProjectSymbolsResponse::Update {
            project_symbols_update_response,
        } => {
            log::info!(
                "updated symbol claim: success={}, symbol_locator_key={}",
                project_symbols_update_response.success,
                project_symbols_update_response.symbol_locator_key
            );
        }
        ProjectSymbolsResponse::Delete {
            project_symbols_delete_response,
        } => {
            log::info!(
                "deleted project symbols: success={}, modules={}, claims={}",
                project_symbols_delete_response.success,
                project_symbols_delete_response.deleted_module_count,
                project_symbols_delete_response.deleted_symbol_count
            );
        }
    }
}
