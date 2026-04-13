use crate::command_executors::project_symbols::project_symbol_store_mutation::{build_unique_symbol_key, save_and_sync_project_symbol_catalog};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_response::ProjectSymbolsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{project_root_symbol::ProjectRootSymbol, project_root_symbol_locator::ProjectRootSymbolLocator};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsCreateRequest {
    type ResponseType = ProjectSymbolsCreateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols create command: {}", error);
                return ProjectSymbolsCreateResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot create rooted symbols without an opened project.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols create command.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(root_locator) = build_root_locator(self) else {
            log::warn!("Project-symbols create request did not provide a valid locator.");
            return ProjectSymbolsCreateResponse::default();
        };
        let trimmed_display_name = self.display_name.trim();

        if trimmed_display_name.is_empty() || self.struct_layout_id.trim().is_empty() {
            log::warn!("Project-symbols create request requires a non-empty display name and type id.");
            return ProjectSymbolsCreateResponse::default();
        }

        let existing_rooted_symbols = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_rooted_symbols()
            .to_vec();
        let symbol_key = build_unique_symbol_key(trimmed_display_name, &existing_rooted_symbols);
        let mut created_symbol = ProjectRootSymbol::new(
            symbol_key.clone(),
            trimmed_display_name.to_string(),
            root_locator,
            self.struct_layout_id.trim().to_string(),
        );
        *created_symbol.get_metadata_mut() = self.metadata.clone();
        opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .get_rooted_symbols_mut()
            .push(created_symbol);

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsCreateResponse::default();
        }

        ProjectSymbolsCreateResponse {
            success: true,
            created_symbol_key: symbol_key,
        }
    }
}

fn build_root_locator(project_symbols_create_request: &ProjectSymbolsCreateRequest) -> Option<ProjectRootSymbolLocator> {
    if let Some(address) = project_symbols_create_request.address {
        return Some(ProjectRootSymbolLocator::new_absolute_address(address));
    }

    let module_name = project_symbols_create_request
        .module_name
        .as_deref()
        .map(str::trim)
        .filter(|module_name| !module_name.is_empty())?;
    let offset = project_symbols_create_request.offset?;

    Some(ProjectRootSymbolLocator::new_module_offset(module_name.to_string(), offset))
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsCreateRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project, project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn create_project_symbol_request_persists_rooted_symbol_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Player Manager"),
            struct_layout_id: String::from("player.manager"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x1234),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);
        assert_eq!(project_symbols_create_response.created_symbol_key, "sym.player.manager");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected created-symbol project to load from disk.");
        let rooted_symbols = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_rooted_symbols();

        assert_eq!(rooted_symbols.len(), 1);
        assert_eq!(rooted_symbols[0].get_display_name(), "Player Manager");
        assert_eq!(rooted_symbols[0].get_struct_layout_id(), "player.manager");
        assert_eq!(
            rooted_symbols[0].get_root_locator(),
            &ProjectRootSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_rooted_symbols(), rooted_symbols);
    }
}
