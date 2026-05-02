use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::update::project_symbols_update_request::ProjectSymbolsUpdateRequest;
use squalr_engine_api::commands::project_symbols::update::project_symbols_update_response::ProjectSymbolsUpdateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsUpdateRequest {
    type ResponseType = ProjectSymbolsUpdateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols update command: {}", error);
                return ProjectSymbolsUpdateResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot update symbol claims without an opened project.");
            return ProjectSymbolsUpdateResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols update command.");
            return ProjectSymbolsUpdateResponse::default();
        };

        let trimmed_display_name = self
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|display_name| !display_name.is_empty())
            .map(str::to_string);
        let trimmed_struct_layout_id = self
            .struct_layout_id
            .as_deref()
            .map(str::trim)
            .filter(|struct_layout_id| !struct_layout_id.is_empty())
            .map(str::to_string);

        if trimmed_display_name.is_none() && trimmed_struct_layout_id.is_none() {
            log::warn!("Project-symbols update request requires at least one non-empty update field.");
            return ProjectSymbolsUpdateResponse::default();
        }

        let Some(symbol_claim) = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .find_symbol_claim_mut(&self.symbol_locator_key)
        else {
            log::warn!(
                "Project-symbols update request could not find symbol locator key '{}'.",
                self.symbol_locator_key
            );
            return ProjectSymbolsUpdateResponse::default();
        };

        if let Some(display_name) = trimmed_display_name {
            symbol_claim.set_display_name(display_name);
        }

        if let Some(struct_layout_id) = trimmed_struct_layout_id {
            symbol_claim.set_struct_layout_id(struct_layout_id);
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsUpdateResponse::default();
        }

        ProjectSymbolsUpdateResponse {
            success: true,
            symbol_locator_key: self.symbol_locator_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpdateRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim};
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn update_project_symbol_request_updates_display_name_and_type_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("i32"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_update_response = ProjectSymbolsUpdateRequest {
            symbol_locator_key: String::from("absolute:1234"),
            display_name: Some(String::from("Player Manager")),
            struct_layout_id: Some(String::from("u64")),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_update_response.success);
        assert_eq!(project_symbols_update_response.symbol_locator_key, "absolute:1234");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected updated-symbol project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_display_name(), "Player Manager");
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u64");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }
}
