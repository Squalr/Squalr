use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_response::ProjectSymbolsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator};
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
            log::warn!("Cannot create symbol claims without an opened project.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols create command.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(locator) = build_locator(self) else {
            log::warn!("Project-symbols create request did not provide a valid locator.");
            return ProjectSymbolsCreateResponse::default();
        };
        let trimmed_display_name = self.display_name.trim();

        if trimmed_display_name.is_empty() || self.struct_layout_id.trim().is_empty() {
            log::warn!("Project-symbols create request requires a non-empty display name and type id.");
            return ProjectSymbolsCreateResponse::default();
        }

        let mut created_symbol = ProjectSymbolClaim::new(trimmed_display_name.to_string(), locator, self.struct_layout_id.trim().to_string());
        let created_symbol_locator_key = created_symbol.get_symbol_locator_key();
        *created_symbol.get_metadata_mut() = self.metadata.clone();
        opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .get_symbol_claims_mut()
            .push(created_symbol);

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsCreateResponse::default();
        }

        ProjectSymbolsCreateResponse {
            success: true,
            created_symbol_locator_key,
        }
    }
}

fn build_locator(project_symbols_create_request: &ProjectSymbolsCreateRequest) -> Option<ProjectSymbolLocator> {
    if let Some(address) = project_symbols_create_request.address {
        return Some(ProjectSymbolLocator::new_absolute_address(address));
    }

    let module_name = project_symbols_create_request
        .module_name
        .as_deref()
        .map(str::trim)
        .filter(|module_name| !module_name.is_empty())?;
    let offset = project_symbols_create_request.offset?;

    Some(ProjectSymbolLocator::new_module_offset(module_name.to_string(), offset))
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
        project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_locator::ProjectSymbolLocator,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn create_project_symbol_request_persists_symbol_claim_and_syncs_catalog() {
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
        assert_eq!(project_symbols_create_response.created_symbol_locator_key, "module:game.exe:1234");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected created-symbol project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_display_name(), "Player Manager");
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "player.manager");
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }
}
